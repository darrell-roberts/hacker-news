use anyhow::Context;
use app::{update, view, App, AppMsg, PaneState, ScrollBy};
use app_dirs2::get_app_dir;
use articles::ArticleState;
use chrono::{DateTime, Utc};
use flexi_logger::{Age, Cleanup, Criterion, FileSpec, Naming};
use footer::FooterState;
use full_search::FullSearchState;
use hacker_news_api::ArticleType;
use hacker_news_search::SearchContext;
use header::{HeaderMsg, HeaderState};
use iced::{
    advanced::graphics::core::window,
    keyboard::{key::Named, on_key_press, Key, Modifiers},
    widget::{
        pane_grid::{self, Configuration},
        text_input::{self, focus},
    },
    window::{close_requests, resize_events},
    Size, Subscription, Task, Theme,
};
#[cfg(target_family = "unix")]
use libc::{getrlimit, rlimit, setrlimit, RLIMIT_NOFILE};
use log::{error, info};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

mod app;
mod articles;
mod comments;
mod config;
mod footer;
mod full_search;
mod header;
mod richtext;
#[cfg(feature = "trace")]
mod tracing;
mod widget;

fn start() -> anyhow::Result<()> {
    let log_dir = get_app_dir(app_dirs2::AppDataType::UserData, &config::APP_INFO, "logs")?;

    let _logger = flexi_logger::Logger::try_with_env_or_str("info")?
        .log_to_file(
            FileSpec::default()
                .directory(log_dir)
                .basename("hacker-news"),
        )
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(5),
        )
        .print_message()
        .start()?;

    #[cfg(target_family = "unix")]
    check_nofiles_limit();

    let index_dir = get_app_dir(
        app_dirs2::AppDataType::UserData,
        &config::APP_INFO,
        "hacker-news-index",
    )?;

    let search_context = Arc::new(RwLock::new(SearchContext::new(
        &index_dir,
        ArticleType::Top,
    )?));

    let app = config::load_config()
        .map(|config| config.into_app(search_context.clone()))
        .unwrap_or_else(|err| {
            error!("Could not load config: {err}");

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
                    current_index_stats: None,
                    index_stats: HashMap::new(),
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
                    full_count: 0,
                },
                focused_pane: None,
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
                    focus(text_input::Id::new("article_search")),
                ]),
            )
        })
        .context("Failed to run UI")
}

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "trace")]
    tracing::init_tracing()?;
    // console_subscriber::init();
    start()
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
            Named::Tab if modifiers.shift() => AppMsg::PrevInput,
            Named::Tab => AppMsg::NextInput,
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

#[cfg(target_family = "unix")]
fn check_nofiles_limit() {
    const DESIRED_LIMIT: u64 = 10_240;

    let mut rlim = rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    unsafe {
        if getrlimit(RLIMIT_NOFILE, &mut rlim) != 0 {
            let errno = *libc::__error();
            error!("Could not get open files limit: {errno}");
            return;
        }
    }

    info!("Current open file limits: {rlim:?}");

    if rlim.rlim_cur < DESIRED_LIMIT {
        rlim.rlim_cur = DESIRED_LIMIT;
        rlim.rlim_max = DESIRED_LIMIT;

        unsafe {
            if setrlimit(RLIMIT_NOFILE, &rlim) != 0 {
                let errno = *libc::__error();
                error!("Could not set open files limit: {errno}");
                return;
            }
        }
        info!("Increased open file limit to {DESIRED_LIMIT}");
    }
}
