// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Error;
use chrono::{DateTime, Local, Utc};
use flexi_logger::{FileSpec, Logger};
use hacker_news_api::{subscribe_top_stories, ApiClient, Item};
use log::{error, info};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tauri::{
    async_runtime::{spawn, TokioJoinHandle},
    Manager, State, Window,
};
use types::{HNItem, PositionChange, TopStories};

mod types;

struct App {
    top_stories: Vec<HNItem>,
    live_events: bool,
    last_event_ids: Vec<u64>,
    event_handle: Option<TokioJoinHandle<()>>,
}

type AppState = Arc<RwLock<App>>;
type AppClient = Arc<ApiClient>;

// #[tauri::command]
// async fn get_item(item_id: u64, state: State<'_, ApiClient>) -> Result<HNItem, String> {
//     state
//         .item(item_id)
//         .await
//         .map(Into::into)
//         .map_err(|err| err.to_string())
// }

#[tauri::command]
fn toggle_live_events(
    app_client: State<'_, AppClient>,
    state: State<'_, AppState>,
    window: Window,
) -> bool {
    let mut s = state.write().unwrap();
    s.live_events = !s.live_events;

    if s.live_events {
        subscribe(window, state.inner().clone(), app_client.inner().clone());
    } else {
        if let Some(h) = s.event_handle.take() {
            h.abort();
        }
    }

    s.live_events
}

#[tauri::command]
async fn get_items(items: Vec<u64>, client: State<'_, AppClient>) -> Result<Vec<HNItem>, String> {
    client
        .items(&items)
        .await
        .map(to_hn_items)
        .map_err(|err| err.to_string())
}

fn to_view_items(app_state: AppState) -> TopStories {
    let s = app_state.read().unwrap();

    TopStories {
        items: s.top_stories.clone(),
        loaded: fetched_time(),
        rust_articles: s
            .top_stories
            .iter()
            .filter_map(|item| item.title.as_deref())
            .filter(|title| has_rust(title))
            .count(),
    }
}

fn difference(before: &[u64], after: &[u64]) -> Vec<u64> {
    if before.is_empty() {
        return Vec::new();
    }

    after
        .into_iter()
        .filter(|n| !before.contains(n))
        .copied()
        .collect()
}

fn position_change(id: &u64, before: &[u64], after: &[u64]) -> PositionChange {
    if before.is_empty() {
        return PositionChange::UnChanged;
    }

    let before_pos = before.iter().position(|n| n == id);
    let after_pos = after.iter().position(|n| n == id);

    match (before_pos, after_pos) {
        (None, None) => PositionChange::UnChanged,
        (None, Some(_)) => PositionChange::Up,
        (Some(_), None) => PositionChange::Down,
        (Some(a), Some(b)) => match a.cmp(&b) {
            std::cmp::Ordering::Less => PositionChange::Down,
            std::cmp::Ordering::Equal => PositionChange::UnChanged,
            std::cmp::Ordering::Greater => PositionChange::Up,
        },
    }
}

fn subscribe(window: Window, app_state: AppState, app_client: AppClient) {
    info!("Starting subscription");
    spawn(async move {
        let (mut rx, handle) = subscribe_top_stories();
        {
            let mut s = app_state.write().unwrap();
            s.event_handle = Some(handle);
        }
        while let Some(event) = rx.recv().await {
            info!("Received top stories event {} items", event.data.len());
            let mut new_items = event.data;
            new_items.truncate(50);

            // Fetch details for new items.
            match app_client.items(&new_items).await {
                Ok(items) => {
                    {
                        let mut s = app_state.write().unwrap();
                        let new_keys = difference(&s.last_event_ids, &new_items);

                        s.top_stories = items
                            .into_iter()
                            .map(HNItem::from)
                            .map(|item| HNItem {
                                new: new_keys.contains(&item.id),
                                position_change: position_change(
                                    &item.id,
                                    &s.last_event_ids,
                                    &new_items,
                                ),
                                ..item
                            })
                            .collect();
                        s.last_event_ids = new_items;
                    }

                    if let Err(err) = window.emit("top_stories", to_view_items(app_state.clone())) {
                        error!("Failed to emit top stories: {err}");
                    };
                }
                Err(err) => {
                    error!("Failed to get new items {err}");
                }
            }
        }
    });
}

pub fn launch() {
    let app_state = Arc::new(RwLock::new(App {
        top_stories: Vec::new(),
        live_events: true,
        last_event_ids: Vec::new(),
        event_handle: None,
    }));
    let app_client = Arc::new(ApiClient::new().unwrap());
    tauri::Builder::default()
        .manage(app_state.clone())
        .manage(app_client.clone())
        .setup(move |app| {
            Logger::try_with_str("info")? // Write all error, warn, and info messages
                // use a simple filename without a timestamp
                .log_to_file(
                    FileSpec::default().suppress_timestamp().directory(
                        app.path_resolver()
                            .app_log_dir()
                            .unwrap_or_else(|| PathBuf::from("")),
                    ),
                )
                // do not truncate the log file when the program is restarted
                .append()
                .start()?;

            let window = app
                .get_window("main")
                .ok_or_else(|| Error::msg("Failed to get main window"))?;
            subscribe(window, app_state, app_client);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_items, toggle_live_events])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn to_hn_items(items: Vec<Item>) -> Vec<HNItem> {
    items.into_iter().map(Into::into).collect()
}

fn parse_date(time: u64) -> Option<String> {
    let duration = DateTime::<Utc>::from_timestamp(time as i64, 0).map(|then| Utc::now() - then)?;

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

fn fetched_time() -> String {
    let now = Local::now();
    format!("Updated {}", now.format("%I:%M%P"))
}

fn has_rust(title: &str) -> bool {
    title
        .to_lowercase()
        .split_ascii_whitespace()
        .any(|word| word.starts_with("rust"))
}
