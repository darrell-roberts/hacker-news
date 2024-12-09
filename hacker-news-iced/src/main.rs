use anyhow::Context;
use app::{update, view, App, AppMsg, PaneState, ScrollBy};
use app_dirs2::get_app_dir;
use articles::{ArticleMsg, ArticleState};
use chrono::{DateTime, Utc};
use footer::FooterState;
use full_search::FullSearchState;
use hacker_news_api::ArticleType;
use hacker_news_search::{document_stats, SearchContext};
use header::HeaderState;
use iced::{
    advanced::graphics::core::window,
    keyboard::{key::Named, on_key_press, Key, Modifiers},
    widget::pane_grid::{self, Configuration},
    window::{close_requests, resize_events},
    Size, Subscription, Theme,
};
use std::{collections::HashSet, sync::Arc};

mod app;
mod articles;
mod comments;
mod config;
mod footer;
mod full_search;
mod header;
mod richtext;
mod widget;

fn main() -> anyhow::Result<()> {
    let dir = get_app_dir(
        app_dirs2::AppDataType::UserData,
        &config::APP_INFO,
        "hacker-news-index",
    )?;

    let have_index = dir.exists();
    let search_context = Arc::new(SearchContext::new(&dir)?);

    let (total_documents, total_comments) = document_stats(&search_context)?;

    let app = config::load_config()
        .map(|config| App {
            search_context: search_context.clone(),
            theme: theme(&config.theme).unwrap_or_default(),
            scale: config.scale,
            header: HeaderState {
                search_context: search_context.clone(),
                article_count: config.article_count,
                article_type: config.article_type,
                building_index: false,
                full_search: None,
            },
            footer: FooterState {
                status_line: String::new(),
                last_update: None,
                scale: config.scale,
                total_comments,
                total_documents,
            },
            article_state: ArticleState {
                search_context: search_context.clone(),
                articles: Vec::new(),
                visited: config.visited,
                search: None,
                viewing_item: None,
                article_limit: config.article_count,
            },
            comment_state: None,
            size: Size::new(config.window_size.0, config.window_size.1),
            panes: pane_grid::State::with_configuration(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Vertical,
                ratio: 0.3,
                a: Box::new(Configuration::Pane(PaneState::Articles)),
                b: Box::new(Configuration::Pane(PaneState::Comments)),
            }),
            full_search_state: FullSearchState {
                search_context: search_context.clone(),
                search: None,
                search_results: Vec::new(),
                offset: 0,
                page: 1,
            },
        })
        .unwrap_or_else(|err| {
            eprintln!("Could not load config: {err}");

            App {
                search_context: search_context.clone(),
                #[cfg(target_os = "linux")]
                theme: Theme::GruvboxDark,
                #[cfg(not(target_os = "linux"))]
                theme: Theme::GruvboxLight,
                scale: 1.,
                header: HeaderState {
                    search_context: search_context.clone(),
                    article_count: 75,
                    article_type: ArticleType::Top,
                    building_index: false,
                    full_search: None,
                },
                footer: FooterState {
                    status_line: String::new(),
                    last_update: None,
                    scale: 1.,
                    total_comments,
                    total_documents,
                },
                article_state: ArticleState {
                    search_context: search_context.clone(),
                    articles: Vec::new(),
                    visited: HashSet::new(),
                    search: None,
                    viewing_item: None,
                    article_limit: 75,
                },
                comment_state: None,
                size: Size::new(800., 600.),
                panes: pane_grid::State::with_configuration(pane_grid::Configuration::Split {
                    axis: pane_grid::Axis::Vertical,
                    ratio: 1.,
                    a: Box::new(Configuration::Pane(PaneState::Articles)),
                    b: Box::new(Configuration::Pane(PaneState::Comments)),
                }),
                full_search_state: FullSearchState {
                    search_context: search_context.clone(),
                    search: None,
                    search_results: Vec::new(),
                    offset: 0,
                    page: 1,
                },
            }
        });

    iced::application("Hacker News", update, view)
        .theme(|app| app.theme.clone())
        .subscription(|_app| {
            Subscription::batch([
                on_key_press(listen_to_key_events),
                close_requests().map(|_event| AppMsg::WindowClose),
                resize_events().map(|(_id, size)| AppMsg::WindowResize(size)),
            ])
        })
        .window(window::Settings {
            size: app.size,
            #[cfg(target_os = "linux")]
            platform_specific: window::settings::PlatformSpecific {
                application_id: "hacker-news".into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .scale_factor(|app| app.scale)
        .run_with(move || {
            let limit = app.header.article_count;
            (
                app,
                if have_index {
                    iced::Task::done(AppMsg::Articles(ArticleMsg::TopStories(limit)))
                } else {
                    iced::Task::none()
                },
            )
        })
        .context("Failed to run UI")
}

fn listen_to_key_events(key: Key, modifiers: Modifiers) -> Option<AppMsg> {
    match key {
        Key::Named(named) => Some(match named {
            Named::Escape => AppMsg::CloseSearch,
            Named::PageUp => AppMsg::ScrollBy(ScrollBy::PageUp),
            Named::PageDown => AppMsg::ScrollBy(ScrollBy::PageDown),
            Named::ArrowUp => AppMsg::ScrollBy(ScrollBy::LineUp),
            Named::ArrowDown => AppMsg::ScrollBy(ScrollBy::LineDown),
            Named::Home => AppMsg::ScrollBy(ScrollBy::Top),
            Named::End => AppMsg::ScrollBy(ScrollBy::Bottom),
            _ => return None,
        }),
        Key::Character(c) => {
            let char = c.chars().next()?;

            Some(match char {
                'f' if modifiers.control() => AppMsg::Comments(comments::CommentMsg::OpenSearch),
                '+' if modifiers.control() => AppMsg::IncreaseScale,
                '-' if modifiers.control() => AppMsg::DecreaseScale,
                '=' if modifiers.control() => AppMsg::ResetScale,
                _ => return None,
            })
        }
        Key::Unidentified => None,
    }
}

/// Extract the duration from a UNIX time and convert duration into a human
/// friendly sentence.
fn parse_date(time: u64) -> Option<String> {
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

fn theme(theme_name: &str) -> Option<Theme> {
    Theme::ALL
        .iter()
        .find(|&theme| theme.to_string() == theme_name)
        .cloned()
}
