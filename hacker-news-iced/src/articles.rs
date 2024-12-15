use crate::{
    app::AppMsg, footer::FooterMsg, full_search::FullSearchMsg, parse_date,
    richtext::SearchSpanIter, widget::hoverable,
};
use hacker_news_search::{api::Story, update_story, SearchContext};
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
    collections::HashSet,
    sync::{Arc, RwLock},
};

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
}

#[derive(Debug, Clone)]
pub enum ArticleMsg {
    TopStories(usize),
    Receive(Vec<Story>),
    Search(String),
    ViewingItem(u64),
    UpdateStory(Story),
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
                .link(AppMsg::FullSearch(FullSearchMsg::Search(format!(
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
                                            .push(widget::tooltip(
                                                widget::button(
                                                    widget::text("↻")
                                                        .shaping(text::Shaping::Advanced),
                                                )
                                                .style(widget::button::text)
                                                .padding(padding::right(5))
                                                .on_press(AppMsg::Articles(
                                                    ArticleMsg::UpdateStory(story.clone()),
                                                )),
                                                widget::container(
                                                    widget::text("Re-Index")
                                                        .color(iced::Color::WHITE),
                                                )
                                                .style(|_| {
                                                    widget::container::Style::default()
                                                        .background(Background::Color(
                                                            iced::Color::BLACK,
                                                        ))
                                                        .border(iced::border::rounded(8))
                                                })
                                                .padding(4),
                                                widget::tooltip::Position::FollowCursor,
                                            ))
                                            .spacing(5),
                                    )
                                    .align_right(Length::Fill)
                                    // .align_x(Horizontal::Right)
                                    .width(Length::Fill),
                                )
                                .spacing(5),
                        )
                        .push(
                            Row::new()
                                .push(
                                    widget::text(format!("🔼{}", story.score))
                                        .shaping(text::Shaping::Advanced),
                                )
                                .push(if story.descendants == 0 {
                                    Element::from(text(""))
                                } else {
                                    Element::from(comments_button)
                                })
                                .push(
                                    widget::button(
                                        widget::text(format!("{}", story.id))
                                            .font(Font {
                                                weight: Weight::Light,
                                                ..Default::default()
                                            })
                                            .size(12),
                                    )
                                    .on_press(AppMsg::Clipboard(format!("{}", story.id)))
                                    .style(widget::button::text)
                                    .padding(0),
                                )
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
                        Err(err) => {
                            Task::done(FooterMsg::Error(err.to_string())).map(AppMsg::Footer)
                        }
                    }
                }
            }
            ArticleMsg::TopStories(limit) => {
                self.article_limit = limit;
                let g = self.search_context.read().unwrap();
                match g.top_stories(limit, 0) {
                    Ok(stories) => Task::done(AppMsg::Articles(ArticleMsg::Receive(stories))),
                    Err(err) => Task::done(AppMsg::Footer(FooterMsg::Error(err.to_string()))),
                }
            }
            ArticleMsg::ViewingItem(story_id) => {
                self.visited.insert(story_id);
                self.viewing_item = Some(story_id);
                Task::done(AppMsg::SaveConfig)
            }
            ArticleMsg::UpdateStory(story) => {
                let category_type = self.search_context.read().unwrap().active_category();
                let limit = self.article_limit;
                Task::perform(
                    update_story(self.search_context.clone(), story, category_type),
                    move |result| match result {
                        Ok(_) => AppMsg::Articles(ArticleMsg::TopStories(limit)),
                        Err(err) => AppMsg::Footer(FooterMsg::Error(err.to_string())),
                    },
                )
            }
        }
    }
}
