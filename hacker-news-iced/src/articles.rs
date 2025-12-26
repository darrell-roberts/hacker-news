//! View and state for viewing top level stories.
use crate::{
    app::AppMsg,
    common::{self, error_task, tooltip, FontExt as _},
    footer::FooterMsg,
    full_search::FullSearchMsg,
    header::HeaderMsg,
    parse_date,
    richtext::SearchSpanIter,
    ROBOTO_FONT,
};
use hacker_news_search::{api::Story, update_story, watch_story, SearchContext, WatchState};
use iced::{
    advanced::image::Handle,
    alignment::{Horizontal, Vertical},
    border::{self},
    padding,
    widget::{self, text, Column, Row},
    Background, Color, Element, Length, Shadow, Task, Theme,
};
use log::info;
use std::{
    collections::{HashMap, HashSet},
    mem,
    ops::Not,
    sync::{Arc, RwLock},
};
use tokio::task::AbortHandle;

/// Abort handles and task handle for a watched story.
pub struct WatchHandles {
    ui_receiver: iced::task::Handle,
    abort_handles: [AbortHandle; 2],
}

impl WatchHandles {
    /// Call abort on all handles.
    fn abort(self) {
        self.ui_receiver.abort();
        self.abort_handles.into_iter().for_each(|h| h.abort());
    }
}

/// Current state of updates for a watched story.
pub struct WatchChange {
    /// Number of new comments since watching.
    new_comments: u64,
    /// Time of oldest comment before watch turned on.
    beyond: u64,
}

pub struct ArticleState {
    pub search_context: Arc<RwLock<SearchContext>>,
    /// Viewing articles
    pub articles: Vec<Story>,
    /// Visited item ids.
    pub visited: HashSet<u64>,
    /// Search
    pub search: Option<String>,
    /// Item comments are being viewed.
    pub viewing_item: Option<u64>,
    /// How many articles to fetch.
    pub article_limit: usize,
    /// Handles for watch stories.
    pub watch_handles: HashMap<u64, WatchHandles>,
    /// Number of changes that have occurred to a watched story.
    pub watch_changes: HashMap<u64, WatchChange>,
    /// Stories being index.
    pub indexing_stories: Vec<u64>,
    /// Filter articles being watched.
    pub filter_watching: bool,
    /// Handle to static rust image.
    pub rust_image: Handle,
}

impl ArticleState {
    /// New article state.
    pub fn new(search_context: Arc<RwLock<SearchContext>>) -> Self {
        Self {
            search_context,
            articles: Vec::new(),
            visited: HashSet::new(),
            search: None,
            viewing_item: None,
            article_limit: 75,
            watch_handles: HashMap::new(),
            watch_changes: HashMap::new(),
            indexing_stories: Vec::new(),
            filter_watching: false,
            rust_image: Handle::from_bytes(RUST_LOGO),
        }
    }

    /// Set visited set.
    pub fn visited(mut self, visited: HashSet<u64>) -> Self {
        self.visited = visited;
        self
    }

    /// Set article limit.
    pub fn article_limit(mut self, article_limit: usize) -> Self {
        self.article_limit = article_limit;
        self
    }
}

#[derive(Debug, Clone)]
pub enum ArticleMsg {
    TopStories(usize),
    Receive(Vec<Story>),
    Search(String),
    ViewingItem(u64),
    UpdateStory(Story),
    WatchStory(Story),
    UnWatchStory(u64),
    StoryUpdated(Story),
    RemoveWatches,
    OpenNew { story_id: u64, beyond: u64 },
    ClearIndexStory(u64),
    CheckHandles,
    ToggleWatchFilter,
    StoryClicked(Story),
}

static RUST_LOGO: &[u8] = include_bytes!("../../assets/rust-logo-32x32.png");

impl ArticleState {
    /// Render the list of top level stories.
    pub fn view<'a>(&'a self, theme: &Theme) -> Element<'a, AppMsg> {
        widget::scrollable(
            Column::with_children(
                self.articles
                    .iter()
                    .filter(|story| {
                        !self.filter_watching || self.watch_changes.contains_key(&story.id)
                    })
                    .map(|article| self.render_article(theme, article)),
            )
            .width(Length::Fill)
            .spacing(10)
            .padding(padding::top(10).bottom(10).left(15).right(25)),
        )
        .height(Length::Fill)
        .id(widget::Id::new("articles"))
        .into()
    }

    fn render_article_title<'a>(&'a self, story: &'a Story) -> iced::Element<'a, AppMsg> {
        let title = widget::rich_text(
            SearchSpanIter::new(&story.title, self.search.as_deref())
                .map(|span| span.link(story.clone()))
                .collect::<Vec<_>>(),
        )
        .on_link_click(|link| AppMsg::Articles(ArticleMsg::StoryClicked(link)));

        match story.url.as_deref() {
            Some(url) => widget::mouse_area(title)
                .on_enter(AppMsg::Footer(FooterMsg::Url(url.to_string())))
                .on_exit(AppMsg::Footer(FooterMsg::NoUrl))
                .into(),
            None => Element::from(title),
        }
    }

    /// Render a single story.
    fn render_article<'a>(&'a self, theme: &Theme, story: &'a Story) -> iced::Element<'a, AppMsg> {
        let by: widget::text::Rich<'a, String, AppMsg> = widget::rich_text([
            widget::span(format!("by {}", story.by))
                .link(story.by.clone())
                .font(ROBOTO_FONT.italic())
                .size(14)
                .color_maybe(widget::text::primary(theme).color),
            widget::span(" "),
            widget::span(parse_date(story.time).unwrap_or_default())
                .font(ROBOTO_FONT.weight_light().italic())
                .size(10)
                .color_maybe(widget::text::primary(theme).color),
        ])
        .on_link_click(|by| AppMsg::Header(HeaderMsg::Search(format!("by:{by}"))));

        let article_id = story.id;

        let content = widget::container(
            Row::new()
                .push(
                    Column::new()
                        .push(
                            Row::new()
                                .push(
                                    widget::container(self.render_article_title(story)).width(
                                        Length::FillPortion(4).enclose(Length::FillPortion(1)),
                                    ),
                                )
                                .push(
                                    widget::container(
                                        Row::new()
                                            .push({
                                                let has_rust = story.title.split(' ').any(|word| {
                                                    word == "Rust"
                                                        || (word.starts_with("Rust")
                                                            && word.len() == 5
                                                            && word
                                                                .chars()
                                                                .last()
                                                                .map(|c| {
                                                                    matches!(
                                                                        c,
                                                                        ',' | '.' | ':' | '?' | '!'
                                                                    )
                                                                })
                                                                .unwrap_or(false))
                                                });
                                                has_rust.then(|| {
                                                    widget::container(
                                                        widget::image(&self.rust_image)
                                                            .content_fit(iced::ContentFit::Contain),
                                                    )
                                                })
                                            })
                                            .push(self.visited.contains(&story.id).then(|| {
                                                widget::container(
                                                    widget::text("✅")
                                                        .shaping(text::Shaping::Advanced),
                                                )
                                            }))
                                            .push(
                                                self.indexing_stories
                                                    .contains(&story.id)
                                                    .not()
                                                    .then(|| {
                                                        tooltip(
                                                            widget::button(
                                                                widget::text("↻").shaping(
                                                                    text::Shaping::Advanced,
                                                                ),
                                                            )
                                                            .style(widget::button::text)
                                                            .padding(padding::right(5))
                                                            .on_press(AppMsg::Articles(
                                                                ArticleMsg::UpdateStory(
                                                                    story.clone(),
                                                                ),
                                                            )),
                                                            "Update",
                                                            widget::tooltip::Position::FollowCursor,
                                                        )
                                                    }),
                                            )
                                            .spacing(5),
                                    )
                                    .align_right(Length::Fill)
                                    .width(Length::FillPortion(1)),
                                )
                                .spacing(5),
                        )
                        .push(
                            Row::new()
                                .push(widget::text!("{}", story.rank))
                                .push((story.ty != "job").then(|| {
                                    widget::text!("🔼{}", story.score)
                                        .shaping(text::Shaping::Advanced)
                                }))
                                .push(if story.descendants == 0 {
                                    Element::from(text(""))
                                } else {
                                    Element::from(
                                        widget::text!("💬{}", story.descendants)
                                            .shaping(text::Shaping::Advanced),
                                    )
                                })
                                .push((story.ty != "job").then(|| {
                                    tooltip(
                                        widget::toggler(self.watch_handles.contains_key(&story.id))
                                            .on_toggle(|toggled| {
                                                AppMsg::Articles(if toggled {
                                                    ArticleMsg::WatchStory(story.clone())
                                                } else {
                                                    ArticleMsg::UnWatchStory(story.id)
                                                })
                                            }),
                                        "Watch",
                                        widget::tooltip::Position::FollowCursor,
                                    )
                                }))
                                .push(
                                    widget::container(by)
                                        .align_x(Horizontal::Right)
                                        .align_y(Vertical::Bottom)
                                        .width(Length::Fill),
                                )
                                .spacing(5),
                        )
                        .spacing(10),
                )
                .align_y(Vertical::Top)
                .spacing(5),
        )
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let color = if self.viewing_item == Some(article_id) {
                palette.secondary.weak.color
            } else {
                palette.primary.weak.color
            };

            let background = self
                .viewing_item
                .and_then(|id| (id == article_id).then_some(Background::Color(color)));

            widget::container::Style {
                border: border::width(0.5).color(color).rounded(8.),
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: iced::Vector { x: 2., y: 2. },
                    blur_radius: 5.,
                },
                background,
                ..Default::default()
            }
        })
        .padding([5, 15])
        .clip(false);

        let clickable = widget::mouse_area(content).on_press(AppMsg::OpenComment {
            article: story.clone(),
            parent_id: story.id,
            comment_stack: Vec::new(),
        });

        widget::stack(
            [
                Some(clickable.into()),
                self.watch_changes
                    .get(&article_id)
                    .filter(|w| w.new_comments > 0)
                    .map(|watch_change| {
                        widget::opaque(
                            widget::container(
                                widget::container(common::tooltip(
                                    widget::button(
                                        widget::text!("{}", watch_change.new_comments)
                                            .color(Color::from_rgb8(255, 255, 153))
                                            .font(ROBOTO_FONT.bold()),
                                    )
                                    .style(widget::button::text)
                                    .on_press(
                                        AppMsg::Articles(ArticleMsg::OpenNew {
                                            story_id: article_id,
                                            beyond: watch_change.beyond,
                                        }),
                                    ),
                                    "Open new",
                                    widget::tooltip::Position::Left,
                                ))
                                .style(|_theme| {
                                    widget::container::background(Color::from_rgba8(255, 0, 0, 0.8))
                                        .border(iced::border::rounded(25))
                                }),
                            )
                            .align_right(Length::Fill)
                            .padding(iced::padding::top(10).right(10)),
                        )
                    }),
            ]
            .into_iter()
            .flatten(),
        )
        .into()
    }

    /// Update the state of the top level story list view
    pub fn update(&mut self, message: ArticleMsg) -> Task<AppMsg> {
        match message {
            ArticleMsg::Receive(articles) => {
                log::debug!("Received {} articles", articles.len());
                self.articles = articles;
                widget::operation::scroll_to::<AppMsg>(
                    widget::Id::new("articles"),
                    // Default::default(),
                    widget::operation::AbsoluteOffset { x: 0.0, y: 0.0 },
                )
            }
            ArticleMsg::Search(input) => {
                if input.is_empty() {
                    self.search = None;
                    // TODO better state management
                    Task::done(AppMsg::Articles(ArticleMsg::TopStories(self.article_limit)))
                } else {
                    self.search = Some(input.clone());
                    let g = self.search_context.read().unwrap();
                    match g.search_stories(&input, self.article_limit, 0) {
                        Ok(stories) => {
                            self.articles = stories;
                            Task::none()
                        }
                        Err(err) => error_task(err),
                    }
                }
            }
            ArticleMsg::TopStories(limit) => {
                self.article_limit = limit;
                match self.search_context.read().unwrap().top_stories(limit, 0) {
                    Ok(stories) => Task::done(AppMsg::Articles(ArticleMsg::Receive(stories))),
                    Err(err) => error_task(err),
                }
            }
            ArticleMsg::ViewingItem(story_id) => {
                self.visited.insert(story_id);
                self.viewing_item = Some(story_id);
                Task::done(AppMsg::SaveConfig)
            }
            ArticleMsg::UpdateStory(story) => {
                let story_id = story.id;
                self.indexing_stories.push(story_id);
                if let Some(handle) = self.watch_handles.remove(&story.id) {
                    handle.abort();
                }
                Task::future(update_story(self.search_context.clone(), story)).then(move |result| {
                    match result {
                        Ok(Some(story)) => {
                            Task::done(ArticleMsg::StoryUpdated(story)).map(AppMsg::Articles)
                        }
                        Ok(None) => clear_index_story_task(story_id),
                        Err(err) => {
                            Task::batch([error_task(err), clear_index_story_task(story_id)])
                        }
                    }
                })
            }
            ArticleMsg::WatchStory(story) => self.watch_story(story),
            ArticleMsg::StoryUpdated(story) => {
                let story_id = story.id;

                if let Some(s) = self.articles.iter_mut().find(|s| s.id == story.id) {
                    if s.descendants < story.descendants {
                        self.watch_changes.entry(story_id).and_modify(|last_new| {
                            last_new.new_comments =
                                story.descendants - (s.descendants - last_new.new_comments)
                        });
                    }
                    s.descendants = story.descendants;
                    s.score = story.score;
                }
                clear_index_story_task(story_id)
            }
            ArticleMsg::UnWatchStory(story_id) => {
                if let Some(handle) = self.watch_handles.remove(&story_id) {
                    handle.abort();
                }
                self.watch_changes.remove(&story_id);
                Task::none()
            }
            ArticleMsg::RemoveWatches => {
                let watches = mem::take(&mut self.watch_handles);
                for handle in watches.into_values() {
                    handle.abort()
                }
                Task::none()
            }
            ArticleMsg::OpenNew { story_id, beyond } => {
                let latest_comment_time = self
                    .search_context
                    .read()
                    .unwrap()
                    .last_comment_age(story_id)
                    .ok()
                    .into_iter()
                    .flatten()
                    .next();

                if let Some(watch_state) = self.watch_changes.get_mut(&story_id) {
                    let previous_last_time = watch_state.beyond;
                    watch_state.beyond = latest_comment_time.unwrap_or(previous_last_time);
                    watch_state.new_comments = 0;
                }

                Task::batch([
                    Task::done(FullSearchMsg::StoryByTime {
                        story_id,
                        beyond: Some(beyond),
                    })
                    .map(AppMsg::FullSearch),
                    Task::done(ArticleMsg::ViewingItem(story_id)).map(AppMsg::Articles),
                ])
            }
            ArticleMsg::ClearIndexStory(story_id) => {
                self.indexing_stories.retain(|id| id != &story_id);
                Task::none()
            }
            ArticleMsg::CheckHandles => {
                let aborted_watchers = self
                    .watch_handles
                    .iter()
                    .filter_map(|(story_id, watch_handle)| {
                        watch_handle
                            .abort_handles
                            .iter()
                            .any(|h| h.is_finished() || watch_handle.ui_receiver.is_aborted())
                            .then_some(*story_id)
                    })
                    .collect::<Vec<_>>();

                let mut re_connect_tasks = Vec::new();

                for story_id in aborted_watchers {
                    if let Some(watch_handler) = self.watch_handles.remove(&story_id) {
                        info!("Reconnecting watch handler for {story_id}");
                        watch_handler.abort();

                        let story = self.search_context.read().unwrap().story(story_id);

                        match story {
                            Ok(story) => {
                                re_connect_tasks.push(self.watch_story(story));
                            }
                            Err(err) => re_connect_tasks.push(error_task(err)),
                        }
                    }
                }

                if re_connect_tasks.is_empty() {
                    Task::none()
                } else {
                    Task::batch(re_connect_tasks)
                }
            }
            ArticleMsg::ToggleWatchFilter => {
                self.filter_watching = !self.filter_watching;
                Task::none()
            }
            ArticleMsg::StoryClicked(story) => {
                let story_id = story.id;
                Task::batch([
                    match story.url.as_deref() {
                        Some(url) => Task::done(AppMsg::OpenLink {
                            url: url.to_string(),
                        }),
                        None => Task::none(),
                    },
                    Task::done(AppMsg::OpenComment {
                        article: story,
                        parent_id: story_id,
                        comment_stack: Vec::new(),
                    }),
                    Task::done(ArticleMsg::ViewingItem(story_id)).map(AppMsg::Articles),
                ])
            }
        }
    }

    /// Turn on a watch on story. This subscribes to updates that originate from server
    /// side events. Each update event will send an `ArticleMsg::StoryUpdated` message.
    fn watch_story(&mut self, story: Story) -> Task<AppMsg> {
        let story_id = story.id;
        let last_comment_age = self
            .search_context
            .read()
            .unwrap()
            .last_comment_age(story_id)
            .unwrap_or_default();
        self.watch_changes.insert(
            story_id,
            WatchChange {
                new_comments: 0,
                beyond: last_comment_age.unwrap_or_default(),
            },
        );
        match watch_story(self.search_context.clone(), story) {
            Ok(WatchState {
                receiver,
                abort_handles,
            }) => {
                let (task, handle) = Task::run(receiver, ArticleMsg::StoryUpdated)
                    .map(AppMsg::Articles)
                    .abortable();
                self.watch_handles.insert(
                    story_id,
                    WatchHandles {
                        ui_receiver: handle,
                        abort_handles,
                    },
                );
                task
            }
            Err(err) => error_task(err),
        }
    }
}

fn clear_index_story_task(story_id: u64) -> Task<AppMsg> {
    Task::done(ArticleMsg::ClearIndexStory(story_id)).map(AppMsg::Articles)
}
