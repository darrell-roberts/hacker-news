//! Application top level state and view.
use crate::{
    articles::{self, ArticleMsg, ArticleState},
    comments::{self, CommentMsg, CommentState, NavStack},
    common::{self, error_task, FontExt as _},
    config::{save_config, Config},
    footer::{self, FooterMsg, FooterState},
    full_search::{FullSearchMsg, FullSearchState, SearchCriteria},
    header::{self, HeaderMsg, HeaderState},
    nav_history::{Content, History, HistoryElement},
    widget::hoverable,
    ROBOTO_FONT,
};
use hacker_news_api::ArticleType;
use hacker_news_search::{
    api::{Comment, Story},
    SearchContext,
};
use iced::{
    // clipboard,
    widget::{
        self, button, container, focus_next, focus_previous, pane_grid, scrollable::AbsoluteOffset,
        text::Shaping, Column,
    },
    Length,
    Size,
    Task,
    Theme,
};
use log::error;
use std::{
    mem,
    sync::{Arc, RwLock},
};

/// Application state.
pub struct App {
    /// Active theme.
    pub theme: Theme,
    /// Scale.
    pub scale: f64,
    /// Header
    pub header: HeaderState,
    /// Article state.
    pub article_state: ArticleState,
    /// Footer
    pub footer: FooterState,
    /// Window size
    pub size: Size,
    /// Pane grid
    pub panes: pane_grid::State<PaneState>,
    /// Search context.
    pub search_context: Arc<RwLock<SearchContext>>,
    /// Pane with focus
    pub focused_pane: Option<widget::pane_grid::Pane>,
    /// Navigation history.
    pub history: Vec<HistoryElement>,
    /// Main content.
    pub content: Content,
}

#[derive(Debug, Copy, Clone)]
pub enum PaneState {
    Articles,
    Content,
}

#[derive(Debug, Copy, Clone)]
pub enum ScrollBy {
    PageUp,
    PageDown,
    LineUp,
    LineDown,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    Header(header::HeaderMsg),
    Articles(articles::ArticleMsg),
    Footer(footer::FooterMsg),
    Comments(comments::CommentMsg),
    OpenComment {
        article: Story,
        parent_id: u64,
        comment_stack: Vec<Comment>,
    },
    OpenLink {
        url: String,
    },
    ChangeTheme(Theme),
    WindowClose,
    IncreaseScale,
    DecreaseScale,
    ResetScale,
    WindowResize(Size),
    ScrollBy(ScrollBy),
    CloseSearch,
    PaneResized(pane_grid::ResizeEvent),
    CommentsClosed,
    ClearVisited,
    FullSearch(FullSearchMsg),
    SaveConfig,
    SwitchIndex {
        category: ArticleType,
        count: usize,
    },
    NextInput,
    PrevInput,
    FocusPane(widget::pane_grid::Pane),
    // Forward,
    Back,
}

pub fn update(app: &mut App, message: AppMsg) -> Task<AppMsg> {
    match message {
        AppMsg::OpenComment {
            article,
            parent_id,
            mut comment_stack,
        } => {
            // Opening first set of comments from an article.
            let item_id = article.id;
            let from_full_search = !comment_stack.is_empty();
            comment_stack.reverse();
            let comments = comment_stack.pop().map(|c| vec![c]).unwrap_or_default();

            let mut nav_stack = vec![NavStack::root()];

            if !comment_stack.is_empty() {
                nav_stack.extend(comment_stack.into_iter().map(|comment| NavStack {
                    comment: Some(comment),
                    offset: 0,
                    page: 1,
                    scroll_offset: None,
                }));
            };

            let should_add_history = match &app.content {
                // We are not opening the same comments for the same story again.
                Content::Comment(comment_state) => comment_state.article.id != item_id,
                // We are opening the first story comments. Only one empty state is added to the root.
                Content::Empty => app.history.is_empty(),
                _ => true,
            };

            let last_content = mem::replace(
                &mut app.content,
                Content::Comment(CommentState {
                    search_context: app.search_context.clone(),
                    article,
                    comments,
                    nav_stack,
                    search: None,
                    oneline: false,
                    page: 1,
                    offset: 0,
                    full_count: 0,
                    parent_id: 0,
                    active_comment_id: None,
                }),
            );

            if should_add_history {
                let history_item = last_content.into_history_element();
                app.history.push(history_item);
            }

            if from_full_search {
                Task::none()
            } else {
                Task::done(CommentMsg::FetchComments {
                    parent_id,
                    parent_comment: None,
                    scroll_to: None,
                })
                .map(AppMsg::Comments)
            }
            .chain(Task::done(HeaderMsg::ClearSearch).map(AppMsg::Header))
            .chain(Task::done(ArticleMsg::ViewingItem(item_id)).map(AppMsg::Articles))
            .chain(Task::done(FullSearchMsg::CloseSearch).map(AppMsg::FullSearch))
        }
        AppMsg::CommentsClosed => {
            // Only clear the content if we are closing from comments.
            if matches!(app.content, Content::Comment(_)) {
                let should_add_history = match &app.content {
                    Content::Empty => app.history.is_empty(),
                    _ => true,
                };
                let last_content = mem::replace(&mut app.content, Content::Empty);
                if should_add_history {
                    app.history.push(last_content.into_history_element());
                }
                app.article_state.viewing_item = None;
            }

            Task::none()
        }
        AppMsg::OpenLink { url } => {
            open::that(url)
                .inspect_err(|err| {
                    error!("Failed to open url {err}");
                })
                .unwrap_or_default();
            Task::none()
        }
        AppMsg::ChangeTheme(theme) => {
            app.theme = theme;
            save_task(app)
        }
        AppMsg::WindowClose => Task::none(),
        AppMsg::IncreaseScale => {
            app.scale += 0.1;
            Task::batch([
                Task::done(FooterMsg::Scale(app.scale)).map(AppMsg::Footer),
                save_task(app),
            ])
        }
        AppMsg::DecreaseScale => {
            let new_scale = app.scale - 0.1;
            let int = new_scale * 100.0;

            if int > 10.0 {
                app.scale = new_scale;
            }
            Task::batch([
                Task::done(FooterMsg::Scale(app.scale)).map(AppMsg::Footer),
                save_task(app),
            ])
        }
        AppMsg::ResetScale => {
            app.scale = 1.0;
            Task::batch([
                Task::done(FooterMsg::Scale(app.scale)).map(AppMsg::Footer),
                save_task(app),
            ])
        }
        AppMsg::Articles(msg) => app.article_state.update(msg),
        AppMsg::Comments(msg) => match &mut app.content {
            Content::Comment(comment_state) => comment_state.update(msg),
            _ => Task::none(),
        },
        AppMsg::Footer(msg) => app.footer.update(msg),
        AppMsg::Header(msg) => app.header.update(msg),
        AppMsg::WindowResize(size) => {
            app.size = size;
            save_task(&*app)
        }
        AppMsg::ScrollBy(scroll_by) => {
            let scroll_id = widget::scrollable::Id::new(match &app.content {
                Content::Comment(_) => "comments",
                Content::Search(_) => "full_search",
                Content::Empty => "articles",
            });
            match scroll_by {
                ScrollBy::PageUp => {
                    widget::scrollable::scroll_by(scroll_id, AbsoluteOffset { x: 0., y: -500. })
                }
                ScrollBy::PageDown => {
                    widget::scrollable::scroll_by(scroll_id, AbsoluteOffset { x: 0., y: 500. })
                }
                ScrollBy::LineUp => {
                    widget::scrollable::scroll_by(scroll_id, AbsoluteOffset { x: 0., y: -10. })
                }
                ScrollBy::LineDown => {
                    widget::scrollable::scroll_by(scroll_id, AbsoluteOffset { x: 0., y: 10. })
                }
                ScrollBy::Top => {
                    widget::scrollable::scroll_to(scroll_id, AbsoluteOffset { x: 0., y: 0. })
                }
                ScrollBy::Bottom => {
                    widget::scrollable::scroll_to(scroll_id, AbsoluteOffset { x: 0., y: f32::MAX })
                }
            }
        }
        AppMsg::CloseSearch => {
            app.article_state.search = None;
            Task::done(ArticleMsg::TopStories(app.header.article_count)).map(AppMsg::Articles)
        }
        AppMsg::PaneResized(p) => {
            app.panes.resize(p.split, p.ratio);
            Task::none()
        }
        AppMsg::ClearVisited => {
            app.article_state.visited.clear();
            save_task(app)
        }
        AppMsg::FullSearch(msg) => match &mut app.content {
            // We are switching to the search content from comments.
            // We'll create the initial search state here.
            Content::Search(full_search_state) => {
                // Check if we are going from either text search or time sort.
                let same_search = match &full_search_state.search {
                    SearchCriteria::Query(_) => matches!(&msg, FullSearchMsg::Search(_)),
                    SearchCriteria::StoryId { .. } => {
                        matches!(&msg, FullSearchMsg::StoryByTime { .. })
                    }
                };

                // Are we creating a new search state?
                let change_search = matches!(
                    &msg,
                    FullSearchMsg::Search(_) | FullSearchMsg::StoryByTime { .. }
                );

                if !same_search && change_search {
                    let search_criteria = full_search_state.search.clone();
                    let last_state = mem::replace(
                        full_search_state,
                        FullSearchState::new(app.search_context.clone(), search_criteria),
                    );
                    app.history
                        .push(HistoryElement::Search(last_state.to_history()));
                }

                full_search_state.update(msg)
            }
            content
                if matches!(
                    msg,
                    FullSearchMsg::Search(_) | FullSearchMsg::StoryByTime { .. }
                ) =>
            {
                app.article_state.viewing_item = match msg {
                    FullSearchMsg::StoryByTime { story_id, .. } => Some(story_id),
                    _ => None,
                };

                let search = match &msg {
                    FullSearchMsg::Search(s) => SearchCriteria::Query(s.to_owned()),
                    FullSearchMsg::StoryByTime { story_id, beyond } => SearchCriteria::StoryId {
                        story_id: *story_id,
                        beyond: beyond.to_owned(),
                    },
                    _ => unreachable!("msg guard for search above"),
                };

                // Create a new search content and re-dispatch message.
                let full_search = FullSearchState {
                    search,
                    search_results: Vec::new(),
                    search_context: app.search_context.clone(),
                    offset: 0,
                    page: 1,
                    full_count: 0,
                };
                let should_add_history = match &content {
                    // We are opening the first story comments. Only one empty state is added to the root.
                    Content::Empty => app.history.is_empty(),
                    _ => true,
                };
                let last_content = mem::replace(content, Content::Search(full_search));
                if should_add_history {
                    app.history.push(last_content.into_history_element());
                }
                Task::done(msg).map(AppMsg::FullSearch)
            }
            _ => Task::none(),
        },
        AppMsg::SaveConfig => save_task(app),
        // AppMsg::Clipboard(s) => clipboard::write(s),
        AppMsg::SwitchIndex { category, count } => {
            let update_history = !matches!(app.content, Content::Empty);
            let last_content = mem::replace(&mut app.content, Content::Empty);
            if update_history {
                app.history.push(last_content.into_history_element());
            }
            let mut g = app.search_context.write().unwrap();
            match g.activate_index(category) {
                Ok(_) => Task::batch([
                    Task::done(FooterMsg::CurrentIndex(category)).map(AppMsg::Footer),
                    Task::done(ArticleMsg::TopStories(count)).map(AppMsg::Articles),
                ]),
                Err(err) => error_task(err),
            }
            // .chain(Task::batch([
            //     // Task::done(AppMsg::CloseSearch),
            //     // Task::done(AppMsg::CommentsClosed),
            //     Task::done(FullSearchMsg::CloseSearch).map(AppMsg::FullSearch),
            // ]))
            .chain(Task::done(AppMsg::SaveConfig))
        }
        AppMsg::NextInput => focus_next(),
        AppMsg::PrevInput => focus_previous(),
        AppMsg::FocusPane(pane) => {
            app.focused_pane = Some(pane);
            Task::none()
        }
        AppMsg::Back => {
            // If we are restoring a full search, put back the search query
            // in the header.
            match app.history.pop() {
                Some(last) => match last.into_content(app.search_context.clone()) {
                    Ok((index, content)) => {
                        app.article_state.viewing_item = content.active_story();
                        app.header.full_search = content.search_text();
                        app.content = content;
                        app.header.article_type = index;

                        Task::done(FooterMsg::CurrentIndex(index)).map(AppMsg::Footer)
                    }
                    Err(err) => common::error_task(err),
                },
                None => Task::none(),
            }
        }
    }
}

/// Render the main view.
pub fn view(app: &App) -> iced::Element<AppMsg> {
    let body = widget::pane_grid(&app.panes, |_pane, state, _is_maximized| {
        let comments_title = || -> Option<iced::Element<AppMsg>> {
            let story = match &app.content {
                Content::Comment(comment_state) => Some(&comment_state.article),
                Content::Search(_) => app.article_state.viewing_item.as_ref().and_then(|id| {
                    app.article_state
                        .articles
                        .iter()
                        .find(|story| story.id == *id)
                }),
                Content::Empty => None,
            }?;

            let title_text = widget::text(&story.title)
                .font(ROBOTO_FONT.bold())
                .shaping(Shaping::Advanced);

            let content: iced::Element<AppMsg> = match story.url.as_deref() {
                Some(url) => hoverable(
                    widget::button(title_text)
                        .on_press(AppMsg::OpenLink {
                            url: url.to_string(),
                        })
                        .style(button::text)
                        .padding(0),
                )
                .on_hover(AppMsg::Footer(FooterMsg::Url(url.to_string())))
                .on_exit(AppMsg::Footer(FooterMsg::NoUrl))
                .into(),
                None => title_text.into(),
            };

            Some(widget::container(content).padding(5).into())
        };

        pane_grid::Content::new(match state {
            PaneState::Articles => app.article_state.view(&app.theme),
            PaneState::Content => match &app.content {
                Content::Comment(comment_state) => comment_state.view(),
                Content::Search(full_search_state) => full_search_state.view(),
                Content::Empty => widget::text("").into(),
            },
        })
        .title_bar(match state {
            PaneState::Articles => pane_grid::TitleBar::new("")
                .controls(pane_grid::Controls::new(
                    widget::Column::new()
                        .push(
                            widget::Row::new()
                                .push(
                                    widget::text_input(
                                        "Search...",
                                        app.article_state.search.as_deref().unwrap_or_default(),
                                    )
                                    .padding(5)
                                    .id(widget::text_input::Id::new("article_search"))
                                    .on_input(|search| {
                                        AppMsg::Articles(ArticleMsg::Search(search))
                                    }),
                                )
                                .push(common::tooltip(
                                    widget::button(widget::text("⟲").shaping(Shaping::Advanced))
                                        .on_press(AppMsg::CloseSearch),
                                    "Clear search",
                                    widget::tooltip::Position::Right,
                                )),
                        )
                        .push(
                            widget::container(common::tooltip(
                                widget::checkbox("Watching", app.article_state.filter_watching)
                                    .on_toggle(|_| AppMsg::Articles(ArticleMsg::ToggleWatchFilter)),
                                "Filter watched",
                                widget::tooltip::Position::Bottom,
                            ))
                            .align_right(Length::Fill),
                        ),
                ))
                .always_show_controls(),
            PaneState::Content => match &app.content {
                // Viewing comments for a story in original order.
                Content::Comment(comment_state) => {
                    pane_grid::TitleBar::new(comments_title().unwrap_or("".into()))
                        .controls(pane_grid::Controls::new(
                            widget::Row::new()
                                .push(common::tooltip(
                                    widget::button(if comment_state.nav_stack.len() > 1 {
                                        "^"
                                    } else {
                                        "X"
                                    })
                                    .on_press(AppMsg::Comments(CommentMsg::PopNavStack)),
                                    if comment_state.nav_stack.len() > 1 {
                                        "Previous comment"
                                    } else {
                                        "Close"
                                    },
                                    widget::tooltip::Position::Bottom,
                                ))
                                .spacing(5),
                        ))
                        .always_show_controls()
                }
                // Viewing comments for a story ordered by time.
                Content::Search(full_search_state)
                    if matches!(full_search_state.search, SearchCriteria::StoryId { .. }) =>
                {
                    pane_grid::TitleBar::new(comments_title().unwrap_or("".into()))
                        .controls(pane_grid::Controls::new(widget::container(
                            widget::Row::new()
                                .push(widget::text(format!("{}", full_search_state.full_count)))
                                .push(
                                    widget::button("X")
                                        .on_press(AppMsg::Header(HeaderMsg::ClearSearch)),
                                )
                                .spacing(5),
                        )))
                        .always_show_controls()
                }
                // Search results for search in all story comments.
                Content::Search(full_search_state) => pane_grid::TitleBar::new(
                    widget::container(
                        widget::text("Searched all comments").font(ROBOTO_FONT.bold()),
                    )
                    .padding(5),
                )
                .controls(pane_grid::Controls::new(widget::container(
                    widget::Row::new()
                        .push(widget::text(format!("{}", full_search_state.full_count)))
                        .push(widget::button("X").on_press(AppMsg::Header(HeaderMsg::ClearSearch)))
                        .spacing(5),
                )))
                .always_show_controls(),
                Content::Empty => pane_grid::TitleBar::new(""),
            },
        })
    })
    .on_resize(10, AppMsg::PaneResized)
    .on_click(AppMsg::FocusPane);

    let main_layout = Column::new()
        .push(app.header.view().map(AppMsg::Header))
        .push(body)
        .push(app.footer.view(&app.theme));

    container(main_layout).into()
}

impl From<&App> for Config {
    fn from(state: &App) -> Self {
        let visited = state.article_state.visited.clone();

        Config {
            scale: state.scale,
            article_count: state.header.article_count,
            article_type: state.header.article_type,
            visited: visited.clone(),
            theme: state.theme.to_string(),
            window_size: (state.size.width, state.size.height),
            current_index_stats: state.footer.current_index_stats,
            index_stats: state.footer.index_stats.values().cloned().collect(),
        }
    }
}

/// Save the current application state into a persistent configuration.
pub fn save_task(app: &App) -> Task<AppMsg> {
    let config = Config::from(app);

    Task::future(save_config(config)).then(|result| match result {
        Ok(_) => Task::none(),
        Err(err) => error_task(err),
    })
}
