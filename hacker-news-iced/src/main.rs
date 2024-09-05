use app::{update, view, App, AppMsg, Showing};
use chrono::{DateTime, Utc};
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
        .scale_factor(|app| app.scale)
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
                    scale: 1.,
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
            let char = c.chars().next()?;

            match char {
                'f' if modifiers.control() => Some(AppMsg::OpenSearch),
                '+' if modifiers.control() => Some(AppMsg::IncreaseScale),
                '-' if modifiers.control() => Some(AppMsg::DecreaseScale),
                '=' if modifiers.control() => Some(AppMsg::ResetScale),
                _ => None,
            }
        }
        Key::Unidentified => None,
    }
}

/// Extract the duration from a UNIX time and convert duration into a human
/// friendly sentence.
pub fn parse_date(time: u64) -> Option<String> {
    let duration =
        DateTime::<Utc>::from_timestamp(time.try_into().ok()?, 0).map(|then| Utc::now() - then)?;

    let hours = duration.num_hours();
    let minutes = duration.num_minutes();
    let days = duration.num_days();

    match (days, hours, minutes) {
        (0, 0, 1) => "1 minute ago".to_string(),
        (0, 0, m) => format!("{m} minutes ago"),
        (0, 1, _) => "1 hour ago".to_string(),
        (0, h, _) => format!("{h} hours ago"),
        (1, _, _) => "1 day ago".to_string(),
        (d, _, _) => format!("{d} days ago"),
    }
    .into()
}
