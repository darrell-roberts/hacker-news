use anyhow::Context;
use app::{update, view, App, AppMsg, PaneState, ScrollBy};
use articles::{ArticleMsg, ArticleState};
use chrono::{DateTime, Utc};
use footer::FooterState;
use hacker_news_api::ArticleType;
#[cfg(target_family = "unix")]
use hacker_news_config::limits::check_nofiles_limit;
use hacker_news_config::{init_logger, search_context};
use hacker_news_search::api_client;
use header::{HeaderMsg, HeaderState};
use iced::{
    advanced::graphics::core::window,
    event::listen_with,
    keyboard::{key::Named, Key, Modifiers},
    time::every,
    widget::pane_grid::{self, Configuration},
    window::{close_requests, resize_events},
    Font, Size, Subscription, Task, Theme,
};
use log::error;
use nav_history::Content;
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use crate::config::load_config;

mod app;
mod articles;
mod comments;
mod common;
mod config;
mod footer;
mod full_search;
mod header;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
mod nav_history;
mod richtext;
#[cfg(feature = "trace")]
mod tracing;
mod widget;

const ROBOTO_FONT: Font = Font::with_name("Roboto");
const ROBOTO_MONO: Font = Font::with_name("Roboto Mono");

fn start() -> anyhow::Result<()> {
    iced::application(
        || {
            let app = create_app().expect("No app");
            let article_type = app.header.article_type;
            let article_count = app.header.article_count;
            (
                app,
                Task::batch([
                    Task::done(HeaderMsg::Select {
                        article_type,
                        article_count,
                    })
                    .map(AppMsg::Header),
                    iced::widget::operation::focus(iced::widget::Id::new("article_search")),
                ]),
            )
        },
        update,
        view,
    )
    .theme(|app: &App| app.theme.clone())
    .subscription(|app| {
        // If we are watching any stories then check periodically on the subscriptions handles.
        let story_handle_watcher = if !app.article_state.watch_handles.is_empty() {
            every(Duration::from_secs(5)).map(|_| AppMsg::Articles(ArticleMsg::CheckHandles))
        } else {
            Subscription::none()
        };

        Subscription::batch([
            listen_with(|event, _, _| {
                if let iced::event::Event::Keyboard(iced::keyboard::Event::KeyReleased {
                    key,
                    modifiers,
                    ..
                }) = event
                {
                    return listen_to_key_events(key, modifiers);
                }
                None
            }),
            close_requests().map(|_event| AppMsg::WindowClose),
            resize_events().map(|(_id, size)| AppMsg::WindowResize(size)),
            story_handle_watcher,
            #[cfg(target_os = "linux")]
            Subscription::run(linux::listen_to_system_changes),
        ])
    })
    .window(window::Settings {
        // size: window_size,
        #[cfg(target_os = "linux")]
        platform_specific: window::settings::PlatformSpecific {
            application_id: "io.github.darrellroberts.hacker-news".into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .scale_factor(|app| app.scale)
    .font(include_bytes!(
        "../../assets/fonts/Roboto-VariableFont_wdth,wght.ttf"
    ))
    .font(include_bytes!(
        "../../assets/fonts/Roboto-Italic-VariableFont_wdth,wght.ttf"
    ))
    .font(include_bytes!(
        "../../assets/fonts/RobotoMono-VariableFont_wght.ttf"
    ))
    .default_font(ROBOTO_FONT)
    .antialiasing(true)
    .run()
    .context("Failed to run UI")
}

fn create_app() -> Result<App, anyhow::Error> {
    let _ = api_client();
    init_logger()?;
    #[cfg(target_family = "unix")]
    check_nofiles_limit();
    let search_context = search_context()?;
    let mut app = load_config()
        .map(|config| config.into_app(search_context.clone()))
        .unwrap_or_else(|err| {
            error!("Could not load config: {err}");

            App {
                search_context: search_context.clone(),
                #[cfg(target_os = "linux")]
                theme: Theme::GruvboxDark,
                #[cfg(not(target_os = "linux"))]
                theme: Theme::GruvboxLight,
                #[cfg(not(target_os = "linux"))]
                scale: 1.,
                #[cfg(target_os = "linux")]
                scale: linux::initial_font_scale(),
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
                    #[cfg(not(target_os = "linux"))]
                    scale: 1.,
                    #[cfg(target_os = "linux")]
                    scale: linux::initial_font_scale(),
                    viewing_index: ArticleType::Top,
                    index_stats: HashMap::new(),
                    index_progress: None,
                },
                article_state: ArticleState {
                    search_context: search_context.clone(),
                    articles: Vec::new(),
                    visited: HashSet::new(),
                    search: None,
                    viewing_item: None,
                    article_limit: 75,
                    watch_handles: HashMap::new(),
                    watch_changes: HashMap::new(),
                    indexing_stories: Vec::new(),
                    filter_watching: false,
                },
                size: Size::new(800., 600.),
                panes: pane_grid::State::with_configuration(pane_grid::Configuration::Split {
                    axis: pane_grid::Axis::Vertical,
                    ratio: 0.3,
                    a: Box::new(Configuration::Pane(PaneState::Articles)),
                    b: Box::new(Configuration::Pane(PaneState::Content)),
                }),
                focused_pane: None,
                content: Content::Empty(ArticleType::Top),
                history: Vec::new(),
            }
        });
    #[cfg(target_os = "linux")]
    {
        use log::info;

        app.scale = linux::initial_font_scale();
        app.footer.scale = app.scale;
        info!("Setting scale to {} from system font scale", app.scale);
        app.theme = linux::initial_theme();
    }
    #[cfg(target_os = "macos")]
    {
        app.theme = macos::initial_theme().unwrap_or(Theme::Light);
    }
    Ok(app)
}

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "trace")]
    tracing::init_tracing()?;
    // console_subscriber::init();
    start()
}

/// Keyboard event subscriptions.
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
            Named::Tab if modifiers.shift() => AppMsg::PrevInput,
            Named::Tab => AppMsg::NextInput,
            _ => return None,
        }),
        Key::Character(c) => {
            let char = c.chars().next()?;

            Some(match char {
                // 'f' if modifiers.control() => AppMsg::Comments(comments::CommentMsg::OpenSearch),
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
