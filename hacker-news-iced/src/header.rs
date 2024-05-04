use hacker_news_api::ArticleType;
use iced::{
    widget::{button, container, row, text},
    Element, Length,
};

use crate::app::{App, AppMsg};

impl App {
    pub fn render_header(&self) -> Element<'_, AppMsg> {
        container(
            row![
                button("Top").on_press(AppMsg::Fetch {
                    limit: self.showing.limit,
                    article_type: ArticleType::Top
                }),
                button("Best").on_press(AppMsg::Fetch {
                    limit: self.showing.limit,
                    article_type: ArticleType::Best
                }),
                button("New").on_press(AppMsg::Fetch {
                    limit: self.showing.limit,
                    article_type: ArticleType::New
                }),
                text("|"),
                // Rule::horizontal(0.1),
                button("Ask").on_press(AppMsg::Fetch {
                    limit: self.showing.limit,
                    article_type: ArticleType::Ask
                }),
                button("Show").on_press(AppMsg::Fetch {
                    limit: self.showing.limit,
                    article_type: ArticleType::Show
                }),
                button("Job").on_press(AppMsg::Fetch {
                    limit: self.showing.limit,
                    article_type: ArticleType::Job
                }),
                text("|"),
                // Rule::horizontal(0.1),
                button("25").on_press(AppMsg::Fetch {
                    limit: 25,
                    article_type: self.showing.article_type
                }),
                button("50").on_press(AppMsg::Fetch {
                    limit: 50,
                    article_type: self.showing.article_type
                }),
                button("75").on_press(AppMsg::Fetch {
                    limit: 75,
                    article_type: self.showing.article_type
                }),
                button("100").on_press(AppMsg::Fetch {
                    limit: 100,
                    article_type: self.showing.article_type
                }),
            ]
            .spacing(10),
        )
        .center_x()
        .width(Length::Fill)
        .into()
    }
}
