//! Application top level state and view.
use crate::{
    articles::{self, ArticleMsg, ArticleState},
    comments::{self, CommentMsg, CommentState, NavStack},
    common::{self, error_task},
    config::{save_config, Config},
    footer::{self, FooterMsg, FooterState},
    full_search::{FullSearchMsg, FullSearchState, SearchCriteria},
    header::{self, HeaderMsg, HeaderState},
    widget::hoverable,
};
use hacker_news_api::ArticleType;
use hacker_news_search::{
    api::{Comment, Story},
    SearchContext,
};
use iced::{
    // clipboard,
    font::Weight,
    widget::{
        self, button, container, focus_next, focus_previous, pane_grid, scrollable::AbsoluteOffset,
        text::Shaping, Column,
    },
    Font,
    Length,
    Size,
    Task,
    Theme,
};
use log::error;
use std::sync::{Arc, RwLock};

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
    /// Comment state.
    pub comment_state: Option<CommentState>,
    /// Full search state.
    pub full_search_state: FullSearchState,
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
}

#[derive(Debug, Copy, Clone)]
pub enum PaneState {
    Articles,
    Comments,
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

            let mut nav_stack = vec![NavStack {
                comment: None,
                offset: 0,
                page: 1,
                scroll_offset: None,
            }];

            if !comment_stack.is_empty() {
                nav_stack.extend(comment_stack.into_iter().map(|comment| NavStack {
                    comment: Some(comment),
                    offset: 0,
                    page: 1,
                    scroll_offset: None,
                }));
            };

            app.comment_state = Some(CommentState {
                search_context: app.search_context.clone(),
                article,
                comments,
                nav_stack,
                search: None,
                oneline: false,
                search_results: Vec::new(),
                page: 1,
                offset: 0,
                full_count: 0,
                parent_id: 0,
            });

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
            app.comment_state = None;
            app.article_state.viewing_item = None;
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
        AppMsg::Comments(msg) => app
            .comment_state
            .as_mut()
            .map(|s| s.update(msg))
            .unwrap_or_else(Task::none),
        AppMsg::Footer(msg) => app.footer.update(msg),
        AppMsg::Header(msg) => app.header.update(msg),
        AppMsg::WindowResize(size) => {
            app.size = size;
            save_task(&*app)
        }
        AppMsg::ScrollBy(scroll_by) => {
            let scroll_id =
                widget::scrollable::Id::new(if app.full_search_state.search.is_some() {
                    "full_search"
                } else if app.comment_state.is_some() {
                    "comments"
                } else {
                    "articles"
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
        AppMsg::FullSearch(msg) => app.full_search_state.update(msg),
        AppMsg::SaveConfig => save_task(app),
        // AppMsg::Clipboard(s) => clipboard::write(s),
        AppMsg::SwitchIndex { category, count } => {
            let mut g = app.search_context.write().unwrap();
            match g.activate_index(category) {
                Ok(_) => Task::batch([
                    Task::done(FooterMsg::CurrentIndex(category)).map(AppMsg::Footer),
                    Task::done(ArticleMsg::TopStories(count)).map(AppMsg::Articles),
                ]),
                Err(err) => error_task(err),
            }
            .chain(Task::batch([
                Task::done(AppMsg::CloseSearch),
                Task::done(AppMsg::CommentsClosed),
                Task::done(FullSearchMsg::CloseSearch).map(AppMsg::FullSearch),
            ]))
            .chain(Task::done(AppMsg::SaveConfig))
        }
        AppMsg::NextInput => focus_next(),
        AppMsg::PrevInput => focus_previous(),
        AppMsg::FocusPane(pane) => {
            app.focused_pane = Some(pane);
            Task::none()
        }
    }
}

/// Render the main view.
pub fn view(app: &App) -> iced::Element<AppMsg> {
    let body = widget::pane_grid(&app.panes, |_pane, state, _is_maximized| {
        let comments_title = || -> Option<iced::Element<AppMsg>> {
            let comment_state = app.comment_state.as_ref()?;
            let title_text = widget::text(&comment_state.article.title)
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                })
                .shaping(Shaping::Advanced);

            let content: iced::Element<AppMsg> = match comment_state.article.url.as_deref() {
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
            PaneState::Comments => {
                if app.full_search_state.search.is_some() {
                    app.full_search_state.view()
                } else {
                    app.comment_state
                        .as_ref()
                        .map(|s| s.view())
                        .unwrap_or_else(|| widget::text("").into())
                }
            }
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
            PaneState::Comments => match app.comment_state.as_ref() {
                // Comment search for selected story
                Some(cs) if app.full_search_state.search.is_none() => {
                    pane_grid::TitleBar::new(comments_title().unwrap_or("".into()))
                        .controls(pane_grid::Controls::new(
                            widget::Row::new()
                                .push(widget::text(format!("{}", cs.full_count)))
                                .push(
                                    widget::toggler(cs.oneline)
                                        .label("oneline")
                                        .on_toggle(|_| AppMsg::Comments(CommentMsg::Oneline)),
                                )
                                .push(widget::button("by time").on_press(AppMsg::FullSearch(
                                    FullSearchMsg::StoryByTime {
                                        story_id: cs.article.id,
                                        beyond: None,
                                    },
                                )))
                                .push(common::tooltip(
                                    widget::button(if cs.nav_stack.len() > 1 { "^" } else { "X" })
                                        .on_press(AppMsg::Comments(CommentMsg::PopNavStack)),
                                    if cs.nav_stack.len() > 1 {
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
                // Search comments for story ordered by time
                _ if matches!(
                    app.full_search_state.search,
                    Some(SearchCriteria::StoryId { .. })
                ) =>
                {
                    pane_grid::TitleBar::new(comments_title().unwrap_or("".into()))
                        .controls(pane_grid::Controls::new(widget::container(
                            widget::Row::new()
                                .push(widget::text(format!(
                                    "{}",
                                    app.full_search_state.full_count
                                )))
                                .push(
                                    widget::button("X")
                                        .on_press(AppMsg::Header(HeaderMsg::ClearSearch)),
                                )
                                .spacing(5),
                        )))
                        .always_show_controls()
                }
                // Regular all comment search
                _ if app.full_search_state.search.is_some() => pane_grid::TitleBar::new(
                    widget::container(widget::text("Searched all comments").font(Font {
                        weight: Weight::Bold,
                        ..Default::default()
                    }))
                    .padding(5),
                )
                .controls(pane_grid::Controls::new(widget::container(
                    widget::Row::new()
                        .push(widget::text(format!(
                            "{}",
                            app.full_search_state.full_count
                        )))
                        .push(widget::button("X").on_press(AppMsg::Header(HeaderMsg::ClearSearch)))
                        .spacing(5),
                )))
                .always_show_controls(),
                _ => pane_grid::TitleBar::new(""),
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
