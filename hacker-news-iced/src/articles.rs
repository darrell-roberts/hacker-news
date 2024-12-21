use crate::{
    app::AppMsg,
    common::{error_task, tooltip},
    footer::FooterMsg,
    full_search::FullSearchMsg,
    header::HeaderMsg,
    parse_date,
    richtext::SearchSpanIter,
    widget::hoverable,
};
use hacker_news_search::{api::Story, update_story, watch_story, SearchContext, WatchState};
use iced::{
    advanced::image::{Bytes, Handle},
    alignment::{Horizontal, Vertical},
    border::{self},
    font::{Style, Weight},
    padding,
    widget::{self, button, scrollable, text, Column, Row},
    Background, Color, Element, Font, Length, Shadow, Task, Theme,
};
use std::{
    collections::{HashMap, HashSet},
    mem,
    ops::Not,
    sync::{Arc, RwLock},
};
use tokio::task::AbortHandle;

pub struct WatchHandles {
    ui_receiver: iced::task::Handle,
    abort_handles: [AbortHandle; 2],
}

impl WatchHandles {
    fn abort(self) {
        self.ui_receiver.abort();
        self.abort_handles.into_iter().for_each(|h| h.abort());
    }
}

pub struct WatchChange {
    new_comments: u64,
    beyond: u64,
}

pub struct ArticleState {
    pub search_context: Arc<RwLock<SearchContext>>,
    /// Viewing articles
    pub articles: Vec<Story>,
    /// Visisted item ids.
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
    FetchStory(u64),
    ClearIndexStory(u64),
}

static RUST_LOGO: Bytes = Bytes::from_static(include_bytes!("../../assets/rust-logo-32x32.png"));

impl ArticleState {
    pub fn view<'a>(&'a self, theme: &Theme) -> Element<'a, AppMsg> {
        widget::scrollable(
            Column::with_children(
                self.articles
                    .iter()
                    .map(|article| self.render_article(theme, article))
                    .map(Element::from),
            )
            .width(Length::Fill)
            .spacing(10)
            .padding(padding::top(10).bottom(10).left(15).right(25)),
        )
        .height(Length::Fill)
        .id(scrollable::Id::new("articles"))
        .into()
    }

    fn render_article<'a>(
        &'a self,
        theme: &Theme,
        story: &'a Story,
    ) -> widget::Container<'a, AppMsg> {
        let title = widget::rich_text(
            SearchSpanIter::new(&story.title, self.search.as_deref())
                .map(|span| {
                    span.link_maybe(
                        story
                            .url
                            .clone()
                            .map(|url| AppMsg::OpenLink {
                                url,
                                item_id: story.id,
                            })
                            .or_else(|| {
                                story.body.as_ref().map(|_| AppMsg::OpenComment {
                                    article: story.clone(),
                                    parent_id: story.id,
                                    comment_stack: Vec::new(),
                                })
                            }),
                    )
                })
                .collect::<Vec<_>>(),
        );

        let by = widget::rich_text([
            widget::span(format!(" by {}", story.by))
                .link(AppMsg::Header(HeaderMsg::Search(format!(
                    "by:{}",
                    story.by
                ))))
                .font(Font {
                    style: Style::Italic,
                    ..Default::default()
                })
                .size(14)
                .color_maybe(widget::text::primary(theme).color),
            widget::span(" "),
            widget::span(parse_date(story.time).unwrap_or_default())
                .font(Font {
                    weight: Weight::Light,
                    style: Style::Italic,
                    ..Default::default()
                })
                .size(10)
                .color_maybe(widget::text::primary(theme).color),
        ]);

        let content = format!("💬{}", story.descendants);

        let comments_button = button(widget::text(content).shaping(text::Shaping::Advanced))
            .style(button::text)
            .padding(0)
            .on_press_maybe((story.descendants > 0).then(|| AppMsg::OpenComment {
                article: story.clone(),
                parent_id: story.id,
                comment_stack: Vec::new(),
            }));

        let title_wrapper = match story.url.as_deref() {
            Some(url) => hoverable(title)
                .on_hover(AppMsg::Footer(FooterMsg::Url(url.to_string())))
                .on_exit(AppMsg::Footer(FooterMsg::NoUrl))
                .into(),
            None => Element::from(title),
        };

        let article_id = story.id;

        widget::container(
            Row::new()
                .push(
                    Column::new()
                        .push(
                            Row::new()
                                .push_maybe(
                                    self.watch_changes
                                        .get(&article_id)
                                        .filter(|w| w.new_comments > 0)
                                        .map(|watch_change| {
                                            widget::container(
                                                widget::button(
                                                    widget::text(format!(
                                                        "+{}",
                                                        watch_change.new_comments
                                                    ))
                                                    .color(Color::from_rgb8(255, 255, 153)),
                                                )
                                                .style(widget::button::text)
                                                .on_press(AppMsg::Articles(ArticleMsg::OpenNew {
                                                    story_id: article_id,
                                                    beyond: watch_change.beyond,
                                                })),
                                            )
                                            .style(
                                                |_theme| {
                                                    widget::container::background(Color::from_rgb8(
                                                        255, 0, 0,
                                                    ))
                                                    .border(iced::border::rounded(25))
                                                },
                                            )
                                        }),
                                )
                                .push(
                                    widget::container(title_wrapper)
                                        .width(Length::FillPortion(4).enclose(Length::Fill)),
                                )
                                .push(
                                    widget::container(
                                        Row::new()
                                            .push_maybe({
                                                let has_rust = story
                                                    .title
                                                    .split(' ')
                                                    .any(|word| word == "Rust");
                                                has_rust.then(|| {
                                                    widget::container(
                                                        widget::image(Handle::from_bytes(
                                                            RUST_LOGO.clone(),
                                                        ))
                                                        .content_fit(iced::ContentFit::Contain),
                                                    )
                                                })
                                            })
                                            .push_maybe(self.visited.contains(&story.id).then(
                                                || {
                                                    widget::container(
                                                        widget::text("✅")
                                                            .shaping(text::Shaping::Advanced),
                                                    )
                                                },
                                            ))
                                            .push_maybe(
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
                                                            "Re-Index",
                                                            widget::tooltip::Position::FollowCursor,
                                                        )
                                                    }),
                                            )
                                            .spacing(5),
                                    )
                                    .align_right(Length::Fill)
                                    .width(Length::Fill),
                                )
                                .spacing(5),
                        )
                        .push(
                            Row::new()
                                .push(widget::text(format!("{}", story.rank)))
                                .push_maybe((story.ty != "job").then(|| {
                                    widget::text(format!("🔼{}", story.score))
                                        .shaping(text::Shaping::Advanced)
                                }))
                                .push(if story.descendants == 0 {
                                    Element::from(text(""))
                                } else {
                                    Element::from(comments_button)
                                })
                                .push_maybe((story.ty != "job").then(|| {
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

            let color = if self
                .viewing_item
                .map(|id| id == article_id)
                .unwrap_or(false)
            {
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
        .clip(false)
    }

    pub fn update(&mut self, message: ArticleMsg) -> Task<AppMsg> {
        match message {
            ArticleMsg::Receive(articles) => {
                self.articles = articles;
                widget::scrollable::scroll_to::<AppMsg>(
                    widget::scrollable::Id::new("articles"),
                    Default::default(),
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
                    match g.search_stories(&input, 0) {
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
                // let watch_handles = mem::take(&mut self.watch_handles);
                // for handle in watch_handles.into_values() {
                //     handle.abort();
                // }
                // self.watch_changes.clear();
                match self.search_context.read().unwrap().top_stories(limit, 0) {
                    Ok(stories) => Task::done(AppMsg::Articles(ArticleMsg::Receive(stories))),
                    Err(err) => error_task(err),
                }
            }
            ArticleMsg::ViewingItem(story_id) => {
                self.visited.insert(story_id);
                self.viewing_item = Some(story_id);
                self.watch_changes.remove(&story_id);
                Task::done(AppMsg::SaveConfig)
            }
            ArticleMsg::UpdateStory(story) => {
                let story_id = story.id;
                self.indexing_stories.push(story_id);
                let category_type = self.search_context.read().unwrap().active_category();
                if let Some(handle) = self.watch_handles.remove(&story.id) {
                    handle.abort();
                }
                Task::future(update_story(
                    self.search_context.clone(),
                    story,
                    category_type,
                ))
                .then(move |result| match result {
                    Ok(_) => Task::done(ArticleMsg::FetchStory(story_id)).map(AppMsg::Articles),
                    Err(err) => Task::batch([error_task(err), clear_index_story_task(story_id)]),
                })
            }
            ArticleMsg::WatchStory(story) => {
                let story_id = story.id;
                let category_type = self.search_context.read().unwrap().active_category();

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

                match watch_story(self.search_context.clone(), story, category_type) {
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

                Task::done(FullSearchMsg::StoryByTime {
                    story_id,
                    beyond: Some(beyond),
                })
                .map(AppMsg::FullSearch)
            }
            ArticleMsg::FetchStory(story_id) => {
                match self.search_context.read().unwrap().story(story_id) {
                    Ok(story) => Task::done(ArticleMsg::StoryUpdated(story)).map(AppMsg::Articles),
                    Err(err) => Task::batch([error_task(err), clear_index_story_task(story_id)]),
                }
            }
            ArticleMsg::ClearIndexStory(story_id) => {
                self.indexing_stories.retain(|id| id != &story_id);
                Task::none()
            }
        }
    }
}

fn clear_index_story_task(story_id: u64) -> Task<AppMsg> {
    Task::done(ArticleMsg::ClearIndexStory(story_id)).map(AppMsg::Articles)
}
