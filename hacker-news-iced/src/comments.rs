use crate::{app::AppMsg, footer::FooterMsg, parse_date, richtext::render_rich_text};
use chrono::Local;
use hacker_news_api::Item;
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
    pub parent: Option<Item>,
    /// Comment items.
    pub items: Vec<Item>,
}

/// Comment state
pub struct CommentState {
    /// Article this comment belongs to
    pub article: Item,
    /// Children
    pub comments: Vec<CommentItem>,
    /// Search
    pub search: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CommentMsg {
    ReceiveComments(Vec<Item>, Option<Item>),
    CloseComment,
    Search(String),
    OpenSearch,
    CloseSearch,
}

impl CommentState {
    pub fn view(&self) -> Element<'_, AppMsg> {
        // let header = row![button("X").on_press(AppMsg::Comments(CommentMsg::CloseComment))];

        let article_text = self
            .article
            .text
            .as_deref()
            .map(render_rich_text)
            .map(|rt| container(rt).padding([10, 10]).into());

        let comment_rows = match self.comments.iter().last() {
            Some(item) => either::Left(
                item.items
                    .iter()
                    .filter(
                        |item| match (self.search.as_deref(), item.text.as_deref()) {
                            (Some(search), Some(text)) => {
                                text.to_lowercase().contains(&search.to_lowercase())
                            }
                            _ => true,
                        },
                    )
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
                    .map(Element::from),
            ),
            None => either::Right(std::iter::empty::<Element<'_, AppMsg>>()),
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

    fn render_comment<'a>(&'a self, item: &'a Item, is_parent: bool) -> Container<'a, AppMsg> {
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
                    comment_ids: item.kids.clone(),
                    parent: Some(item.clone()),
                })
                .style(button::text)
                .into()
        };

        container(
            column![
                render_rich_text(item.text.as_deref().unwrap_or_default()),
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
                    self.search = None;
                } else {
                    self.search = Some(search);
                }
                Task::none()
            }
            CommentMsg::OpenSearch => {
                self.search = Some(String::new());
                widget::text_input::focus(widget::text_input::Id::new("comment_search"))
            }
            CommentMsg::CloseSearch => {
                self.search = None;
                Task::none()
            }
        }
    }
}
