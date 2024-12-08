use std::sync::Arc;

use crate::{app::AppMsg, footer::FooterMsg, parse_date, richtext::render_rich_text};
use chrono::Local;
use hacker_news_search::{
    stories::{Comment, Story},
    SearchContext,
};
use iced::{
    border,
    font::{Style, Weight},
    padding,
    widget::{self, button, column, container, row, scrollable, text::Shaping, Column, Container},
    Border, Element, Font, Length, Task,
};

/// List of comments and common parent
pub struct CommentItem {
    /// Parent comment.
    pub parent: Option<Comment>,
    /// Comment items.
    pub items: Vec<Comment>,
}

/// Comment state
pub struct CommentState {
    pub search_context: Arc<SearchContext>,
    /// Article this comment belongs to
    pub article: Story,
    /// Children
    pub comments: Vec<CommentItem>,
    /// Search
    pub search: Option<String>,
    /// Show one line only.
    pub oneline: bool,

    pub search_results: Vec<Comment>,
}

#[derive(Debug, Clone)]
pub enum CommentMsg {
    ReceiveComments(Vec<Comment>, Option<Comment>),
    CloseComment,
    Search(String),
    OpenSearch,
    CloseSearch,
    Oneline,
}

impl CommentState {
    pub fn view(&self) -> Element<'_, AppMsg> {
        let article_text = self
            .article
            .body
            .as_deref()
            .map(|text| widget::rich_text(render_rich_text(text, self.search.as_deref(), false)))
            .map(|rt| container(rt).padding([10, 10]).into());

        let comment_rows = if !self.search_results.is_empty() {
            self.search_results
                .iter()
                .map(|comment| {
                    self.render_comment(comment, false).style(|theme| {
                        let palette = theme.extended_palette();

                        container::Style {
                            background: Some(palette.background.weak.color.into()),
                            border: border::rounded(8),
                            ..Default::default()
                        }
                    })
                })
                .map(Element::from)
                .collect::<Vec<_>>()
        } else {
            self.comments
                .iter()
                .last()
                .into_iter()
                .flat_map(|item| {
                    item.items
                        .iter()
                        .filter(|item| match self.search.as_deref() {
                            Some(search) => {
                                item.body.to_lowercase().contains(&search.to_lowercase())
                            }
                            _ => true,
                        })
                        .map(|item| {
                            self.render_comment(item, false).style(|theme| {
                                let palette = theme.extended_palette();

                                container::Style {
                                    background: Some(palette.background.weak.color.into()),
                                    border: border::rounded(8),
                                    ..Default::default()
                                }
                            })
                        })
                        .map(Element::from)
                })
                .collect::<Vec<_>>()
        };

        let parent_comments = self.comments.iter().flat_map(|item| {
            item.parent
                .as_ref()
                .map(|parent| {
                    self.render_comment(parent, true).style(|theme| {
                        let palette = theme.extended_palette();

                        container::Style {
                            border: Border {
                                color: palette.secondary.weak.color,
                                width: 1.,
                                radius: 8.into(),
                            },
                            ..Default::default()
                        }
                    })
                })
                .map(Element::from)
        });

        let content = Column::new()
            .push_maybe(self.search.as_ref().map(|search| {
                widget::Row::new()
                    .push(
                        widget::text_input("Search...", search)
                            .id(widget::text_input::Id::new("comment_search"))
                            .on_input(|input| AppMsg::Comments(CommentMsg::Search(input))),
                    )
                    .push(widget::tooltip(
                        widget::button(widget::text("⟲").shaping(Shaping::Advanced))
                            .on_press(AppMsg::Comments(CommentMsg::CloseSearch)),
                        widget::container(widget::text("Clear search")).padding(5),
                        widget::tooltip::Position::Left,
                    ))
            }))
            .push(
                scrollable(
                    column![
                        Column::with_children(article_text).spacing(15),
                        Column::with_children(parent_comments).spacing(15),
                        Column::with_children(comment_rows).spacing(15)
                    ]
                    .spacing(15)
                    .padding(padding::top(0).bottom(10).left(10).right(25)),
                )
                .id(widget::scrollable::Id::new("comments"))
                .height(Length::Fill),
            );

        container(content.width(Length::Fill)).into()
    }

    fn render_comment<'a>(&'a self, item: &'a Comment, is_parent: bool) -> Container<'a, AppMsg> {
        let by_button: Element<'_, AppMsg> = if item.kids.is_empty() {
            widget::text("").into()
        } else if is_parent {
            widget::text(format!("💬{}", item.kids.len()))
                .shaping(Shaping::Advanced)
                .into()
        } else {
            button(widget::text(format!("💬{}", item.kids.len())).shaping(Shaping::Advanced))
                .padding(0)
                .on_press(AppMsg::OpenComment {
                    article: None,
                    parent_id: item.id,
                    parent: Some(item.clone()),
                })
                .style(button::text)
                .into()
        };

        container(
            column![
                widget::rich_text(render_rich_text(
                    &item.body,
                    self.search.as_deref(),
                    self.oneline
                )),
                row![
                    widget::rich_text([
                        widget::span(format!(" by {}", item.by))
                            .font(Font {
                                style: Style::Italic,
                                ..Default::default()
                            })
                            .size(14),
                        widget::span(" "),
                        widget::span(parse_date(item.time).unwrap_or_default())
                            .font(Font {
                                weight: Weight::Light,
                                style: Style::Italic,
                                ..Default::default()
                            })
                            .size(10),
                    ]),
                    by_button,
                ]
                .spacing(5)
            ]
            .padding([10, 10])
            .spacing(15)
            .width(Length::Fill),
        )
        .clip(false)
    }

    pub fn update(&mut self, message: CommentMsg) -> Task<AppMsg> {
        match message {
            CommentMsg::ReceiveComments(comments, parent) => {
                self.comments.push(CommentItem {
                    items: comments,
                    parent,
                });

                Task::done(FooterMsg::LastUpdate(Local::now())).map(AppMsg::Footer)
            }
            CommentMsg::CloseComment => {
                self.comments.pop();
                if self.comments.is_empty() {
                    Task::done(AppMsg::CommentsClosed)
                } else {
                    Task::none()
                }
            }
            CommentMsg::Search(search) => {
                if search.is_empty() {
                    Task::done(AppMsg::Comments(CommentMsg::CloseSearch))
                } else {
                    self.search = Some(search.clone());
                    match self
                        .search_context
                        .search_comments(&search, self.article.id, 0)
                    {
                        Ok(comments) => {
                            self.search_results = comments;
                        }
                        Err(err) => {
                            eprintln!("Failed search: {err}");
                            return Task::done(AppMsg::Footer(FooterMsg::Error(err.to_string())));
                        }
                    }
                    Task::none()
                }
            }
            CommentMsg::OpenSearch => {
                self.search = Some(String::new());
                widget::text_input::focus(widget::text_input::Id::new("comment_search"))
            }
            CommentMsg::CloseSearch => {
                self.search = None;
                self.search_results = Vec::new();
                Task::none()
            }
            CommentMsg::Oneline => {
                self.oneline = !self.oneline;
                Task::none()
            }
        }
    }
}
