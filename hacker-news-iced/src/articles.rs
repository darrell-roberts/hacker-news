use crate::app::{App, AppMsg};
use hacker_news_api::Item;
use iced::{
    alignment::Vertical,
    font::Style,
    widget::{self, button, row, scrollable, text, tooltip::Position, Column, Tooltip},
    Background, Border, Color, Element, Font, Length, Theme,
};
use std::ops::Not;

impl App {
    pub fn render_articles<'a>(&'a self) -> Element<'a, AppMsg> {
        let article_row = |article: &'a Item| {
            let title_and_by = widget::rich_text([
                widget::span(
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
                        .then(|| Color::from_rgb8(122, 122, 82)),
                ),
                widget::span(format!(" by {}", article.by))
                    .font(Font {
                        style: Style::Italic,
                        ..Default::default()
                    })
                    .line_height(0.5)
                    .color(if matches!(self.theme, Theme::GruvboxLight) {
                        Color::from_rgb8(153, 77, 0)
                    } else {
                        Color::from_rgb8(255, 221, 128)
                    }),
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

            let tooltip = match article.url.as_deref() {
                Some(url) => Tooltip::new(
                    title_and_by,
                    widget::container(widget::text(url).color(Color::WHITE).size(12))
                        .padding(2)
                        .style(|_theme| widget::container::Style {
                            background: Some(Background::Color(Color::BLACK)),
                            border: Border {
                                radius: 2.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    Position::Bottom,
                )
                .into(),
                None => Element::from(title_and_by),
            };

            row![
                widget::text(format!("🔼{}", article.score))
                    .width(55)
                    .shaping(text::Shaping::Advanced),
                if article.kids.is_empty() {
                    Element::from(text("").width(55))
                } else {
                    Element::from(comments_button)
                },
                tooltip
            ]
            .align_y(Vertical::Center)
            .spacing(5)
        };

        scrollable(
            Column::with_children(self.articles.iter().map(article_row).map(Element::from))
                .width(Length::Fill)
                .padding(10),
        )
        .height(Length::Fill)
        .into()
    }
}
