// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Error;
use commands::*;
use flexi_logger::{FileSpec, Logger};
use hacker_news_api::ApiClient;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tauri::{async_runtime::TokioJoinHandle, Manager};
use types::HNItem;

mod commands;
mod types;

/// Application state.
struct App {
    /// Current top stories.
    top_stories: Vec<HNItem>,
    /// Live events enabled.
    live_events: bool,
    /// Last set of event ids.
    last_event_ids: Vec<u64>,
    /// Event handle for background task.
    event_handle: Option<TokioJoinHandle<()>>,
    /// Number of articles.
    total_articles: usize,
}

/// Application State.
type AppState = Arc<RwLock<App>>;

/// Application API Client.
type AppClient = Arc<ApiClient>;

/// Configure and launch the Tauri desktop application.
pub fn launch() {
    let app_state = Arc::new(RwLock::new(App {
        top_stories: Vec::new(),
        live_events: true,
        last_event_ids: Vec::new(),
        event_handle: None,
        total_articles: 75,
    }));
    let app_client = Arc::new(ApiClient::new().unwrap());
    tauri::Builder::default()
        .manage(app_state.clone())
        .manage(app_client.clone())
        .setup(move |app| {
            Logger::try_with_str("info")?
                .format(log_format)
                .log_to_file(
                    FileSpec::default().suppress_timestamp().directory(
                        app.path_resolver()
                            .app_log_dir()
                            .unwrap_or_else(|| PathBuf::from("")),
                    ),
                )
                .append()
                .start()?;

            let window = app
                .get_window("main")
                .ok_or_else(|| Error::msg("Failed to get main window"))?;
            subscribe(window, app_state, app_client);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_items,
            toggle_live_events,
            get_user,
            update_total_articles
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn log_format(
    write: &mut dyn std::io::Write,
    now: &mut flexi_logger::DeferredNow,
    record: &log::Record,
) -> std::io::Result<()> {
    write!(
        write,
        "{} {} [{}] {}",
        now.format_rfc3339(),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        record.args()
    )
}
