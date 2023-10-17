// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{DateTime, Local, Utc};
use hacker_news_api::{subscribe_top_stories, ApiClient, EventData, Item};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tauri::{async_runtime::spawn, Manager, State, Window};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HNItem {
    pub id: u64,
    pub kids: Vec<u64>,
    pub text: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub score: u64,
    pub time: Option<String>,
    pub by: String,
    pub has_rust: bool,
    pub viewed: bool,
}

impl From<Item> for HNItem {
    fn from(item: Item) -> Self {
        Self {
            id: item.id,
            kids: item.kids,
            text: item.text,
            url: item.url,
            score: item.score,
            time: parse_date(item.time),
            by: item.by,
            has_rust: item.title.as_deref().map(has_rust).unwrap_or(false),
            title: item.title,
            viewed: false,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ViewItems {
    items: Vec<HNItem>,
    loaded: String,
    rust_articles: usize,
    top_stories: HashMap<u64, HNItem>,
}

struct App {
    // app_client: ApiClient,
    view_items: ViewItems,
}

type AppState = Arc<Mutex<App>>;

// #[tauri::command]
// async fn get_stories(state: State<'_, ApiClient>) -> Result<ViewItems, String> {
//     state
//         .top_stories(50)
//         .await
//         .map(to_hn_items)
//         .map(|items| ViewItems {
//             rust_articles: items.iter().filter(|item| item.has_rust).count(),
//             items,
//             loaded: fetched_time(),
//         })
//         .map_err(|err| err.to_string())
// }

// #[tauri::command]
// async fn get_item(item_id: u64, state: State<'_, ApiClient>) -> Result<HNItem, String> {
//     state
//         .item(item_id)
//         .await
//         .map(Into::into)
//         .map_err(|err| err.to_string())
// }

#[tauri::command]
async fn get_items(items: Vec<u64>, state: State<'_, ApiClient>) -> Result<Vec<HNItem>, String> {
    state
        .items(items)
        .await
        .map(to_hn_items)
        // .map(|items| ViewItems {
        //     rust_articles: items.iter().filter(|item| item.has_rust).count(),
        //     items,
        //     loaded: fetched_time(),
        // })
        .map_err(|err| err.to_string())
}

fn main() {
    let app_state = Arc::new(Mutex::new(App {
        view_items: ViewItems {
            items: Vec::new(),
            loaded: "".to_string(),
            rust_articles: 0,
            top_stories: HashMap::new(),
        },
    }));
    let app_client = Arc::new(ApiClient::new().unwrap());
    tauri::Builder::default()
        .manage(app_state.clone())
        .manage(app_client.clone())
        .setup(move |app| {
            let window = app.get_window("main").unwrap();
            spawn(async move {
                let mut rx = subscribe_top_stories();
                while let Some(data) = rx.recv().await {
                    let new_items = {
                        let s = app_state.lock().unwrap();

                        data.data
                            .iter()
                            .copied()
                            .filter(|id| !s.view_items.top_stories.contains_key(id))
                            .map(|id| id as u64)
                            .collect::<Vec<_>>()
                    };

                    match app_client.items(new_items).await {
                        Ok(items) => {
                            let mut new_items_map = items
                                .into_iter()
                                .map(|item| (item.id, item))
                                .collect::<HashMap<_, _>>();
                            let mut s = app_state.lock().unwrap();

                            let new_top_stories = data
                                .data
                                .iter()
                                .copied()
                                .filter_map(|id| {
                                    new_items_map
                                        .remove(&id)
                                        .map(Into::into)
                                        .or_else(|| s.view_items.top_stories.remove(&id))
                                })
                                .map(|item| (item.id, item))
                                .collect::<HashMap<_, _>>();

                            s.view_items.top_stories = new_top_stories;

                            if let Err(err) = window.emit(
                                "top_stories",
                                s.view_items.top_stories.values().collect::<Vec<_>>(),
                            ) {
                                eprintln!("Failed to emit top stories: {err}");
                            };
                        }
                        Err(err) => {
                            eprintln!("Failed to get new items {err}");
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            //     get_stories,
            //     get_item,
            get_items,
            //     // subscribe
        ])
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
    format!("Loaded {}", now.format("%I:%M%P"))
}

fn has_rust(title: &str) -> bool {
    title
        .to_lowercase()
        .split_ascii_whitespace()
        .any(|word| word.starts_with("rust"))
}
