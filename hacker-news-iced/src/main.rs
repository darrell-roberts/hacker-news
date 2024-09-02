use app::{update, view, App, AppMsg, Showing};
use hacker_news_api::{ApiClient, ArticleType};
use iced::Theme;
use std::{collections::HashSet, sync::Arc};

mod app;
mod articles;
mod comment;
mod footer;
mod header;
mod richtext;

fn main() -> iced::Result {
    iced::application("Hacker News", update, view)
        .theme(|app| app.theme.clone())
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
                    theme: Theme::GruvboxLight,
                },
                iced::Task::perform(
                    async move { client.articles(75, ArticleType::Top).await },
                    AppMsg::Receive,
                ),
            )
        })
}
