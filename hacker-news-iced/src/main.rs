use app::{update, view, App, AppMsg, Showing};
use hacker_news_api::{ApiClient, ArticleType};
use iced::Theme;
use std::{collections::HashSet, sync::Arc};

mod app;
mod articles;
mod comment;
mod header;
mod richtext;

fn main() -> iced::Result {
    iced::application("Hacker News", update, view)
        .theme(|_| Theme::GruvboxDark)
        .run_with(|| {
            let client = Arc::new(ApiClient::new().expect("Valid client"));

            (
                App {
                    articles: Vec::new(),
                    client: client.clone(),
                    showing: Showing {
                        limit: 75,
                        article_type: ArticleType::Top,
                    },
                    status_line: String::new(),
                    comments: None,
                    visited: HashSet::new(),
                },
                iced::Task::perform(
                    async move { client.articles(75, ArticleType::Top).await },
                    AppMsg::Receive,
                ),
            )
        })
}
