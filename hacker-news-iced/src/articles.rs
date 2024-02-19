use crate::widget::link::text_link;
use hacker_news_api::Item;
use iced::{
    font::Style,
    widget::{button, row, scrollable, text, Column},
    Element, Font, Length, Theme,
};

use crate::app::{App, AppMsg};

impl App {
    pub fn render_articles(&self) -> Element<'_, AppMsg> {
        let article_row = |(article, index): (&Item, usize)| {
            row![
                text(format!("{index}")).width(30),
                text(format!("🔼{}", article.score))
                    .width(50)
                    .shaping(text::Shaping::Advanced),
                if article.kids.is_empty() {
                    Element::from(text("").width(40))
                } else {
                    Element::from(
                        text_link(format!("💬{}", article.kids.len()))
                            .shaping(text::Shaping::Advanced)
                            .width(40)
                            .on_press(AppMsg::OpenComment {
                                article: Some(article.clone()),
                                comment_ids: article.kids.clone(),
                                parent: None,
                            }),
                    )
                },
                text_link(article.title.as_deref().unwrap_or_default()).on_press(
                    match article.url.as_deref() {
                        Some(url) => AppMsg::OpenLink(url.to_string()),
                        None => AppMsg::OpenComment {
                            article: Some(article.clone()),
                            comment_ids: article.kids.clone(),
                            parent: None,
                        },
                    }
                ),
                text(format!("by {}", article.by)).font(Font {
                    style: Style::Italic,
                    ..Default::default()
                })
            ]
            .spacing(5)
        };

        scrollable(
            Column::with_children(
                self.articles
                    .iter()
                    .zip(1..)
                    .map(article_row)
                    .map(Element::from),
            )
            .width(Length::Fill)
            .padding(10),
        )
        .height(Length::Fill)
        .into()
    }
}

pub struct CommentsButtonStyle;

impl button::StyleSheet for CommentsButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance::default()
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            shadow_offset: active.shadow_offset + iced::Vector::new(0.0, 1.0),
            ..active
        }
    }
}
