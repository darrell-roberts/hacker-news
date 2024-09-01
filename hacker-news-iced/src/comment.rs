use crate::{
    app::{App, AppMsg},
    richtext::render_rich_text,
    widget::link::text_link,
};
use hacker_news_api::Item;
use iced::{
    alignment::{Horizontal, Vertical},
    font::{Style, Weight},
    theme::Container,
    widget::{button, column, container, row, scrollable, text, text::Shaping, Column},
    Border, Color, Element, Font, Length, Shadow, Theme,
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

        let comment_row =
            |item: &'a Item,
             stylesheet: Box<dyn container::StyleSheet<Style = Theme> + 'static>| {
                container(
                    column![
                        column(render_rich_text(item.text.as_deref().unwrap_or_default())),
                        row![
                            text(format!("by {}", item.by)).font(Font {
                                style: Style::Italic,
                                ..Default::default()
                            }),
                            if item.kids.is_empty() {
                                Element::from(text(""))
                            } else {
                                Element::from(
                                    text_link(format!("💬{}", item.kids.len()))
                                        .on_press(AppMsg::OpenComment {
                                            article: None,
                                            comment_ids: item.kids.clone(),
                                            parent: Some(item.clone()),
                                        })
                                        .shaping(Shaping::Advanced),
                                )
                            }
                        ]
                        .spacing(5)
                    ]
                    .padding(10)
                    .spacing(5)
                    .width(Length::Fill),
                )
                .clip(false)
                .style(Container::Custom(stylesheet))
            };

        let comment_rows = comment_state.comments.iter().last().map(|item| {
            item.items
                .iter()
                .map(|item| comment_row(item, Box::new(CommentBubbleStyle)))
                .map(Element::from)
                .collect::<Vec<_>>()
        });

        let parent_comments = comment_state
            .comments
            .iter()
            .flat_map(|item| {
                item.parent
                    .as_ref()
                    .map(|parent| comment_row(parent, Box::new(ParentCommentBubbleStyle)))
                    .map(Element::from)
            })
            .collect::<Vec<_>>();

        let content = column![
            row![
                container(
                    text_link(comment_state.article.title.as_deref().unwrap_or_default())
                        .font(Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        })
                        .vertical_alignment(Vertical::Bottom)
                )
                .align_y(Vertical::Bottom),
                container(header)
                    .align_x(Horizontal::Right)
                    .width(Length::Fill)
            ]
            .padding([5, 10, 0, 10]),
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

pub struct CommentBubbleStyle;

impl container::StyleSheet for CommentBubbleStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            background: Some(iced::Background::Color(Color {
                r: 102. / 255.,
                g: 75. / 255.,
                b: 0.,
                a: 1.,
            })),
            // background: Some(iced::Background::Color(Color {
            //     r: 248. / 255.,
            //     g: 222. / 255.,
            //     b: 126. / 255.,
            //     a: 1.,
            // })),
            border: Border {
                color: Color::WHITE,
                width: 1.,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub struct ParentCommentBubbleStyle;

impl container::StyleSheet for ParentCommentBubbleStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: None,
            // background: Some(iced::Background::Color(Color {
            //     r: 1.,
            //     g: 1.,
            //     b: 200. / 255.,
            //     a: 1.,
            // })),
            border: Border {
                color: Color::WHITE,
                width: 1.,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    }
}
