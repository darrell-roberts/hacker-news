use crate::{app::AppMsg, footer::FooterMsg, parse_date, widget::hoverable};
use chrono::Local;
use hacker_news_api::{ApiClient, ArticleType, Item};
use iced::{
    advanced::image::{Bytes, Handle},
    alignment::{Horizontal, Vertical},
    border,
    font::{Style, Weight},
    padding,
    widget::{self, button, scrollable, text, Column, Row},
    Color, Element, Font, Length, Shadow, Task, Theme,
};
use std::{collections::HashSet, ops::Not, sync::Arc};

pub struct ArticleState {
    /// API Client.
    pub client: Arc<ApiClient>,
    /// Viewing articles
    pub articles: Vec<Item>,
    /// Visisted item ids.
    pub visited: HashSet<u64>,
    /// Search
    pub search: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ArticleMsg {
    Fetch {
        limit: usize,
        article_type: ArticleType,
    },
    Receive(Vec<Item>),
    Search(String),
    Visited(u64),
}

static RUST_LOGO: Bytes = Bytes::from_static(include_bytes!("../../assets/rust-logo-32x32.png"));

impl ArticleState {
    pub fn view<'a>(&'a self, theme: &Theme) -> Element<'a, AppMsg> {
        let score_width = self
            .articles
            .iter()
            .map(|article| article.score)
            .max()
            .map_or(0, |max| {
                let digits = max.ilog10() / 10_u64.ilog10();

                match digits {
                    5 => 85,
                    4 => 75,
                    3 => 65,
                    2 => 55,
                    _ => 45,
                }
            });

        let total_comments: usize = self.articles.iter().map(|article| article.kids.len()).sum();

        let article_row = |article: &'a Item| {
            let title = widget::rich_text([widget::span(
                article
                    .title
                    .as_ref()
                    .map_or_else(String::new, |s| s.to_owned()),
            )
            .link_maybe(
                article
                    .url
                    .clone()
                    .map(|url| AppMsg::OpenLink {
                        url,
                        item_id: article.id,
                    })
                    .or_else(|| {
                        article.text.as_ref().map(|_| AppMsg::OpenComment {
                            article: Some(article.clone()),
                            comment_ids: article.kids.clone(),
                            parent: None,
                        })
                    }),
            )
            .color_maybe(
                self.visited
                    .contains(&article.id)
                    .then(|| widget::text::secondary(theme).color)
                    .flatten(),
            )]);

            let by = widget::rich_text([
                widget::span(format!(" by {}", article.by))
                    .font(Font {
                        style: Style::Italic,
                        ..Default::default()
                    })
                    .size(14)
                    .color_maybe(widget::text::primary(theme).color),
                widget::span(" "),
                widget::span(parse_date(article.time).unwrap_or_default())
                    .font(Font {
                        weight: Weight::Light,
                        style: Style::Italic,
                        ..Default::default()
                    })
                    .size(10)
                    .color_maybe(widget::text::primary(theme).color),
            ]);

            let content = format!("💬{}", article.kids.len());
            let comments_button = button(widget::text(content).shaping(text::Shaping::Advanced))
                .width(55)
                .style(button::text)
                .padding(0)
                .on_press_maybe(article.kids.is_empty().not().then(|| AppMsg::OpenComment {
                    article: Some(article.clone()),
                    comment_ids: article.kids.clone(),
                    parent: None,
                }));

            let title_wrapper = match article.url.as_deref() {
                Some(url) => hoverable(title)
                    .on_hover(AppMsg::Footer(FooterMsg::Url(url.to_string())))
                    .on_exit(AppMsg::Footer(FooterMsg::NoUrl))
                    .into(),
                None => Element::from(title),
            };

            widget::container(
                Row::new()
                    .push(
                        widget::text(format!("🔼{}", article.score))
                            .width(score_width)
                            .shaping(text::Shaping::Advanced),
                    )
                    .push(if article.kids.is_empty() {
                        Element::from(text("").width(if total_comments == 0 { 0 } else { 55 }))
                    } else {
                        Element::from(comments_button)
                    })
                    .push_maybe({
                        let has_rust = article
                            .title
                            .as_ref()
                            .map(|t| t.split(' ').any(|word| word == "Rust"))
                            .unwrap_or(false);
                        has_rust.then(|| {
                            widget::image(Handle::from_bytes(RUST_LOGO.clone()))
                                .content_fit(iced::ContentFit::ScaleDown)
                        })
                    })
                    .push(title_wrapper)
                    .push(
                        widget::container(by)
                            .align_x(Horizontal::Right)
                            .width(Length::Fill),
                    )
                    .align_y(Vertical::Center)
                    .spacing(5),
            )
            .width(Length::Fill)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                widget::container::Style {
                    border: border::width(0.5)
                        .color(palette.secondary.weak.color)
                        .rounded(8.),
                    shadow: Shadow {
                        color: Color::BLACK,
                        offset: iced::Vector { x: 2., y: 2. },
                        blur_radius: 5.,
                    },
                    ..Default::default()
                }
            })
            .padding([5, 15])
            .clip(false)
        };

        widget::scrollable(
            Column::with_children(
                self.articles
                    .iter()
                    .filter(|article| match self.search.as_deref() {
                        Some(search) => article
                            .title
                            .as_ref()
                            .map(|t| t.to_lowercase().contains(&search.to_lowercase()))
                            .unwrap_or(true),
                        None => true,
                    })
                    .map(article_row)
                    .map(Element::from),
            )
            .width(Length::Fill)
            .spacing(5)
            .padding(padding::top(0).bottom(10).left(15).right(25)),
        )
        .height(Length::Fill)
        .id(scrollable::Id::new("articles"))
        .into()
    }

    pub fn update(&mut self, message: ArticleMsg) -> Task<AppMsg> {
        match message {
            ArticleMsg::Fetch {
                limit,
                article_type,
            } => {
                let client = self.client.clone();
                Task::batch([
                    Task::done(FooterMsg::Fetching).map(AppMsg::Footer),
                    Task::perform(
                        async move { client.articles(limit, article_type).await },
                        |resp| match resp {
                            Ok(articles) => AppMsg::Articles(ArticleMsg::Receive(articles)),
                            Err(err) => AppMsg::Footer(FooterMsg::Error(err.to_string())),
                        },
                    ),
                ])
            }
            ArticleMsg::Receive(articles) => {
                self.articles = articles;
                Task::batch([
                    widget::scrollable::scroll_to::<AppMsg>(
                        widget::scrollable::Id::new("articles"),
                        Default::default(),
                    ),
                    Task::done(FooterMsg::LastUpdate(Local::now())).map(AppMsg::Footer),
                ])
            }
            ArticleMsg::Search(input) => {
                if input.is_empty() {
                    self.search = None;
                } else {
                    self.search = Some(input);
                }
                Task::none()
            }
            ArticleMsg::Visited(index) => {
                self.visited.insert(index);
                Task::none()
            }
        }
    }
}
