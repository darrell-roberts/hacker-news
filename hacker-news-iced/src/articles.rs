use crate::app::{App, AppMsg};
use hacker_news_api::Item;
use iced::{
    font::Style,
    widget::{self, button, row, scrollable, text, Column},
    Element, Font, Length,
};
use std::ops::Not;

impl App {
    pub fn render_articles(&self) -> Element<'_, AppMsg> {
        let article_row = |article: &Item| {
            let title_and_by = widget::rich_text([
                widget::span(
                    article
                        .title
                        .as_ref()
                        .map_or_else(String::new, |s| s.to_owned()),
                )
                .link_maybe(article.url.clone().map(|url| AppMsg::OpenLink {
                    url,
                    item_id: article.id,
                }))
                .color_maybe(
                    self.visited
                        .contains(&article.id)
                        .then(|| [122. / 255., 122. / 255., 82. / 255.]),
                ),
                widget::span(format!(" by {}", article.by))
                    .font(Font {
                        style: Style::Italic,
                        ..Default::default()
                    })
                    .line_height(0.5)
                    .color([1., 221. / 255., 128. / 255.]),
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

            row![
                widget::text(format!("🔼{}", article.score))
                    .width(55)
                    .shaping(text::Shaping::Advanced),
                if article.kids.is_empty() {
                    Element::from(text("").width(55))
                } else {
                    Element::from(comments_button)
                },
                title_and_by
            ]
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
