use crate::{
    app::{App, AppMsg},
    richtext::render_rich_text,
    // widget::link::text_link,
};
use hacker_news_api::Item;
use iced::{
    alignment::{Horizontal, Vertical},
    font::{Style, Weight},
    widget::{self, button, column, container, row, scrollable, text::Shaping, Column},
    Border, Color, Element, Font, Length,
};

pub struct CommentItem {
    pub parent: Option<Item>,
    pub items: Vec<Item>,
}

pub struct CommentState {
    pub article: Item,
    pub comments: Vec<CommentItem>,
}

impl App {
    pub fn render_comments<'a>(&self, comment_state: &'a CommentState) -> Element<'a, AppMsg> {
        let header = row![button("X").on_press(AppMsg::CloseComment)];

        let comment_row = |item: &'a Item, is_parent: bool| {
            let by_button: Element<'a, AppMsg> = if item.kids.is_empty() {
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
                        widget::text(format!("by {}", item.by))
                            .font(Font {
                                style: Style::Italic,
                                ..Default::default()
                            })
                            .color([1., 221. / 255., 128. / 255.]),
                        by_button
                    ]
                    .spacing(5)
                ]
                .padding(10)
                .spacing(5)
                .width(Length::Fill),
            )
            .clip(false)
        };

        let comment_rows = comment_state.comments.iter().last().map(|item| {
            item.items
                .iter()
                .map(|item| {
                    comment_row(item, false).style(|_theme| container::Style {
                        border: Border {
                            color: Color::BLACK,
                            width: 1.,
                            radius: 8.into(),
                        },
                        background: Some(iced::Background::Color(Color {
                            r: 102. / 255.,
                            g: 75. / 255.,
                            b: 0.,
                            a: 1.,
                        })),
                        ..Default::default()
                    })
                })
                .map(Element::from)
                .collect::<Vec<_>>()
        });

        let parent_comments = comment_state
            .comments
            .iter()
            .flat_map(|item| {
                item.parent
                    .as_ref()
                    .map(|parent| {
                        comment_row(parent, true).style(|_theme| container::Style {
                            border: Border {
                                color: Color::WHITE,
                                width: 1.,
                                radius: 8.into(),
                            },
                            ..Default::default()
                        })
                    })
                    .map(Element::from)
            })
            .collect::<Vec<_>>();

        let content = column![
            row![
                container(
                    widget::text(comment_state.article.title.as_deref().unwrap_or_default()).font(
                        Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        }
                    ) // .vertical_alignment(Vertical::Bottom)
                )
                .align_y(Vertical::Bottom),
                container(header)
                    .align_x(Horizontal::Right)
                    .width(Length::Fill)
            ]
            .padding([5, 10]),
            scrollable(
                column![
                    Column::with_children(parent_comments).spacing(10),
                    Column::with_children(comment_rows.unwrap_or_default()).spacing(10)
                ]
                .spacing(10)
                .padding(10)
            )
            .height(Length::Fill)
        ];

        container(content.width(Length::Fill)).into()
    }
}
