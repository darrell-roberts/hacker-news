use crate::{
    app::{App, AppMsg},
    parse_date,
    richtext::render_rich_text, // widget::link::text_link,
};
use hacker_news_api::Item;
use iced::{
    alignment::{Horizontal, Vertical},
    font::{Style, Weight},
    widget::{self, button, column, container, row, scrollable, text::Shaping, Column, Container},
    Border, Element, Font, Length, Padding,
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
}

impl App {
    pub fn render_comments<'a>(&'a self, comment_state: &'a CommentState) -> Element<'a, AppMsg> {
        let header = row![button("X").on_press(AppMsg::CloseComment)];

        let article_text = comment_state
            .article
            .text
            .as_deref()
            .map(render_rich_text)
            .map(|rt| container(rt).padding([10, 10]).into());

        let comment_rows = match comment_state.comments.iter().last() {
            Some(item) => either::Left(
                item.items
                    .iter()
                    .map(|item| {
                        self.render_comment(item, false)
                            .style(container::rounded_box)
                    })
                    .map(Element::from),
            ),
            None => either::Right(std::iter::empty::<Element<'_, AppMsg>>()),
        };

        let parent_comments = comment_state.comments.iter().flat_map(|item| {
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

        let content = column![
            row![
                container(
                    widget::text(comment_state.article.title.as_deref().unwrap_or_default()).font(
                        Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        }
                    )
                )
                .align_y(Vertical::Bottom),
                container(header)
                    .align_x(Horizontal::Right)
                    .width(Length::Fill)
            ]
            .padding([5, 10]),
            scrollable(
                column![
                    Column::with_children(article_text).spacing(10),
                    Column::with_children(parent_comments).spacing(10),
                    Column::with_children(comment_rows).spacing(10)
                ]
                .spacing(10)
                .padding(Padding::from([0, 10]).right(20))
            )
            .height(Length::Fill)
        ];

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
                            // .line_height(0.5)
                            .size(14)
                            .color_maybe(widget::text::primary(&self.theme).color),
                        widget::span(" "),
                        widget::span(parse_date(item.time).unwrap_or_default())
                            .font(Font {
                                weight: Weight::Light,
                                style: Style::Italic,
                                ..Default::default()
                            })
                            // .line_height(0.5)
                            .size(10)
                            .color_maybe(widget::text::primary(&self.theme).color),
                    ]),
                    by_button,
                ]
                .spacing(5)
            ]
            .padding([10, 10])
            .spacing(5)
            .width(Length::Fill),
        )
        .clip(false)
    }
}
