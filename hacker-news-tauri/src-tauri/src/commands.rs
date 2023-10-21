//! Tauri commands bridging the UI with the backend.
use crate::types::{has_rust, HNItem, HNUser, PositionChange, TopStories};
use crate::{AppClient, AppState};
use anyhow::Context;
use chrono::Local;
use hacker_news_api::{subscribe_top_stories, Item, ResultExt};
use log::{error, info};
use tauri::{async_runtime::spawn, State, Window};

/// Enables or disables the live feed. The background task is terminated when stopped
/// and started when enabling the live feed.
#[tauri::command]
pub(crate) fn toggle_live_events(
    app_client: State<'_, AppClient>,
    state: State<'_, AppState>,
    window: Window,
) -> bool {
    let mut s = state.write().unwrap();
    s.live_events = !s.live_events;

    if s.live_events {
        subscribe(window, state.inner().clone(), app_client.inner().clone());
    } else if let Some(h) = s.event_handle.take() {
        info!("Stopping subscription task");
        h.abort();
    }

    s.live_events
}

/// Get hacker news items. Can be comments, jobs, stories...
#[tauri::command]
pub(crate) async fn get_items(
    items: Vec<u64>,
    client: State<'_, AppClient>,
) -> Result<Vec<HNItem>, String> {
    client
        .items(&items)
        .await
        .map(to_hn_items)
        .map_err(|err| err.to_string())
}

/// Get a hacker news user.
#[tauri::command]
pub(crate) async fn get_user(
    handle: String,
    client: State<'_, AppClient>,
) -> Result<HNUser, String> {
    client
        .user(&handle)
        .await
        .map(Into::into)
        .map_err(|err| err.to_string())
}

/// Transform API [`Item`]s into [HNItem] view items.
fn to_hn_items(items: Vec<Item>) -> Vec<HNItem> {
    items.into_iter().map(Into::into).collect()
}

fn fetched_time() -> String {
    let now = Local::now();
    format!("Updated {}", now.format("%I:%M%P"))
}

/// Create the [`TopStories`] view from the application state.
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

/// Naive difference between two array slices.
fn difference(before: &[u64], after: &[u64]) -> Vec<u64> {
    if before.is_empty() {
        return Vec::new();
    }

    after
        .iter()
        .filter(|n| !before.contains(n))
        .copied()
        .collect()
}

/// Determine an item position change.
fn position_change(id: &u64, before: &[u64], after_pos: usize) -> PositionChange {
    if before.is_empty() {
        return PositionChange::UnChanged;
    }

    let before_pos = before.iter().position(|n| n == id);

    match (before_pos, after_pos) {
        (None, _) => PositionChange::Up,
        (Some(a), b) => match a.cmp(&b) {
            std::cmp::Ordering::Less => PositionChange::Down,
            std::cmp::Ordering::Equal => PositionChange::UnChanged,
            std::cmp::Ordering::Greater => PositionChange::Up,
        },
    }
}

/// Start a task that subscribes to top stories and emits events
/// back to the main window.
pub(crate) fn subscribe(window: Window, app_state: AppState, app_client: AppClient) {
    info!("Starting subscription");
    spawn(async move {
        let (mut rx, handle) = subscribe_top_stories();
        {
            let mut s = app_state.write().unwrap();
            s.event_handle = Some(handle);
        }

        while let Some(event) = rx.recv().await {
            // info!("Received top stories event {} items", event.data.len());
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
                            .enumerate()
                            .map(|(index, item)| HNItem {
                                new: new_keys.contains(&item.id),
                                position_change: position_change(
                                    &item.id,
                                    &s.last_event_ids,
                                    index,
                                ),
                                ..item
                            })
                            .collect();
                        s.last_event_ids = new_items;
                    }

                    window
                        .emit("top_stories", to_view_items(app_state.clone()))
                        .context("Failed to emit to stories")
                        .log_error_consume();
                }
                Err(err) => {
                    error!("Failed to get new items {err}");
                }
            }
        }
    });
}
