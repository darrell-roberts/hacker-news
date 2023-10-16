// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{DateTime, Local, Utc};
use hacker_news_api::{ApiClient, Item};
use serde::Serialize;
use tauri::State;

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
}

struct App {
    app_client: ApiClient,
}

#[tauri::command]
async fn get_stories(state: State<'_, App>) -> Result<ViewItems, String> {
    state
        .app_client
        .top_stories(50)
        .await
        .map(to_hn_items)
        .map(|items| ViewItems {
            rust_articles: items.iter().filter(|item| item.has_rust).count(),
            items,
            loaded: fetched_time(),
        })
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn get_item(item_id: u64, state: State<'_, App>) -> Result<HNItem, String> {
    state
        .app_client
        .item(item_id)
        .await
        .map(Into::into)
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn get_items(items: Vec<u64>, state: State<'_, App>) -> Result<ViewItems, String> {
    state
        .app_client
        .items(items)
        .await
        .map(to_hn_items)
        .map(|items| ViewItems {
            rust_articles: items.iter().filter(|item| item.has_rust).count(),
            items,
            loaded: fetched_time(),
        })
        .map_err(|err| err.to_string())
}

fn main() {
    let app = App {
        app_client: ApiClient::new().unwrap(),
    };
    tauri::Builder::default()
        .manage(app)
        .invoke_handler(tauri::generate_handler![get_stories, get_item, get_items])
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
