use anyhow::Context;
use app::{update, view, App, AppMsg, ContentScreen};
use articles::{ArticleMsg, ArticleState};
use chrono::{DateTime, Utc};
use footer::{FooterMsg, FooterState};
use hacker_news_api::{ApiClient, ArticleType};
use header::{HeaderMsg, HeaderState};
use iced::{
    advanced::graphics::core::window,
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
mod widget;

fn main() -> anyhow::Result<()> {
    let client = Arc::new(ApiClient::new().context("Could not create api client")?);

    iced::application("Hacker News", update, view)
        .theme(|app| app.theme.clone())
        .subscription(|_app| {
            Subscription::batch([
                on_key_press(listen_to_key_events),
                close_requests().map(|_event| AppMsg::WindowClose),
            ])
        })
        .window(window::Settings {
            platform_specific: window::settings::PlatformSpecific {
                application_id: "hacker-news".into(),
            },
            ..Default::default()
        })
        .scale_factor(|app| app.scale)
        .run_with(|| {
            (
                App {
                    client: client.clone(),
                    #[cfg(target_os = "linux")]
                    theme: Theme::GruvboxDark,
                    #[cfg(not(target_os = "linux"))]
                    theme: Theme::GruvboxLight,
                    scale: 1.,
                    header: HeaderState {
                        article_count: 75,
                        article_type: ArticleType::Top,
                        search: None,
                    },
                    content: ContentScreen::Articles(ArticleState {
                        client: client.clone(),
                        articles: Vec::new(),
                        visited: HashSet::new(),
                        search: None,
                    }),
                    footer: FooterState {
                        status_line: String::new(),
                        last_update: None,
                        scale: 1.,
                    },
                    article_state: None,
                },
                iced::Task::perform(
                    async move { client.articles(75, ArticleType::Top).await },
                    |result| match result {
                        Ok(articles) => AppMsg::Articles(ArticleMsg::Receive(articles)),
                        Err(err) => AppMsg::Footer(FooterMsg::Error(err.to_string())),
                    },
                ),
            )
        })
        .context("Failed to run UI")
}

fn listen_to_key_events(key: Key, modifiers: Modifiers) -> Option<AppMsg> {
    match key {
        Key::Named(named) => {
            matches!(named, Named::Escape).then_some(AppMsg::Header(HeaderMsg::CloseSearch))
        }
        Key::Character(c) => {
            let char = c.chars().next()?;

            match char {
                'f' if modifiers.control() => Some(AppMsg::Header(HeaderMsg::OpenSearch)),
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
