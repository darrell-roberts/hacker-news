use app::{update, view, App, AppMsg, Showing};
use hacker_news_api::{ApiClient, ArticleType};
use iced::{
    keyboard::{key::Named, on_key_press, Key, Modifiers},
    window::close_requests,
    Subscription, Theme,
};
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
        .subscription(|_app| {
            Subscription::batch([
                on_key_press(listen_to_key_events),
                close_requests().map(|_event| AppMsg::WindowClose),
            ])
        })
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
                    search: None,
                    all_articles: Vec::new(),
                },
                iced::Task::perform(
                    async move { client.articles(75, ArticleType::Top).await },
                    AppMsg::Receive,
                ),
            )
        })
}

fn listen_to_key_events(key: Key, modifiers: Modifiers) -> Option<AppMsg> {
    match key {
        Key::Named(named) => matches!(named, Named::Escape).then_some(AppMsg::CloseSearch),
        Key::Character(c) => {
            (modifiers.control() && c.chars().any(|c| c == 'f')).then_some(AppMsg::OpenSearch)
        }
        Key::Unidentified => None,
    }
}
