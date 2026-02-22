//! Main content view
use crate::{ApiClientState, ArticleSelection, article::ArticleView};
use async_compat::Compat;
use futures::{SinkExt, StreamExt, TryStreamExt as _, channel};
use gpui::{App, AppContext, Entity, EventEmitter, ListState, Window, div, prelude::*, px};
use hacker_news_api::{ArticleType, Item, subscribe_to_article_list};
use log::{error, info};
use std::collections::HashMap;

// Main content view.
pub struct ContentView {
    /// List of article view entities currently displayed.
    articles: Vec<Entity<ArticleView>>,
    /// State for scrolling and alignment of the article list.
    list_state: ListState,
    /// Tracks the ranking of each article so that if it moves up
    /// or down we can show by how much.
    article_ranks: HashMap<u64, usize>,
    /// Tracks the number of comments for an article so that when it
    /// changes we can show a visual indicator.
    article_comment_counts: HashMap<u64, u64>,
    /// Subscription to server side events is online.
    pub online: bool,
    /// Handle to the background task that updates articles.
    pub background_task: Option<gpui::Task<()>>,
    /// Sender channel for pushing article updates from background to foreground.
    pub article_sender: Option<channel::mpsc::Sender<Result<Vec<Item>, BackGroundError>>>,
    /// The number of times we have refresh due to an http server side event.
    pub background_refresh_count: usize,
    // We are viewing comments.
    // pub viewing_comments: bool,
}

/// Events emitted by the ContentView to signal UI updates or errors.
pub enum ContentEvent {
    /// Indicates the total number of articles currently displayed.
    TotalArticles(usize),
    // Indicates whether the user is currently viewing article comments.
    // ViewingComments(bool),
    /// Indicates the total number of refreshes due to background updates.
    TotalRefreshes(usize),
    /// Indicates an error, optionally containing an error message.
    Error(Option<String>),
    /// Check if we need to restart background.
    Terminated(ArticleType),
    /// Toggle online status
    OnlineToggle(bool),
}

impl EventEmitter<ContentEvent> for ContentView {}

impl ContentView {
    /// Create a new content view.
    ///
    /// # Arguments
    ///
    /// * `_window` - A mutable reference to the window instance.
    /// * `app` - A mutable reference to the application instance.
    ///
    /// # Returns
    ///
    /// Returns an `Entity<Self>` representing the newly created content view.
    pub fn new(_window: &mut Window, app: &mut App) -> Entity<Self> {
        let entity_content = app.new(|cx: &mut Context<Self>| {
            cx.subscribe_self(|content_view, event, cx| match event {
                ContentEvent::OnlineToggle(enable) => {
                    // There is no change here.
                    if content_view.online == *enable {
                        return;
                    }

                    if *enable {
                        restart_background_task(content_view, cx);
                    } else if let Some(task) = content_view.background_task.take() {
                        drop(task);
                        content_view.online = false;
                    }
                }
                ContentEvent::Terminated(terminated_category) => {
                    let current_category = cx.global::<ArticleSelection>().viewing_article_type;
                    info!("Terminated: {terminated_category} current: {current_category}");
                    if terminated_category == &current_category {
                        // We need to restart.
                        info!("Restarting background subscription for {terminated_category}");
                        restart_background_task(content_view, cx);
                    }
                }
                ContentEvent::Error(_)
                | ContentEvent::TotalArticles(_)
                | ContentEvent::TotalRefreshes(_) => (),
            })
            .detach();

            let list_state = ListState::new(0, gpui::ListAlignment::Top, px(5.0));

            Self {
                list_state,
                articles: Default::default(),
                article_ranks: Default::default(),
                online: false,
                background_task: None,
                article_sender: None,
                article_comment_counts: Default::default(),
                background_refresh_count: 0,
                // viewing_comments: false,
            }
        });

        let background_task = start_background_subscriptions(app, &entity_content);
        entity_content.update(app, |content_view, _ctx| {
            content_view.background_task = Some(background_task);
            content_view.online = true;
        });
        entity_content
    }
}

/// Restarts the background task by dropping the current task and replacing it with a new one.
fn restart_background_task(content_view: &mut ContentView, cx: &mut Context<'_, ContentView>) {
    if let Some(tx) = content_view.article_sender.as_ref()
        && !tx.is_closed()
    {
        let task = start_background_article_list_subscription(cx, tx.clone());
        content_view.background_task.replace(task);
        content_view.online = true;
    }
}

#[derive(Debug, Clone)]
/// Background event subscription errors.
pub enum BackGroundError {
    /// The background task for a given category has been terminated.
    Terminated(ArticleType),
    /// A Running event failed to update existing items with provided error message.
    EventFailed(String),
}

/// Starts a background task that subscribes to the top stories stream,
/// fetches article data, and updates the Content entity accordingly.
///
/// This function is intended to be called once when initializing the Content view.
///
/// This function sets up two asynchronous tasks:
/// 1. One task listens for new batches of articles from a channel and updates
///    the Content entity's articles, list state, and ranking map, unless the
///    user is currently viewing comments.
/// 2. Another task subscribes to the top stories stream, fetches the latest
///    articles using the API client, and sends them through the channel to be
///    processed by the first task.
///
/// # Arguments
///
/// * `app` - A mutable reference to the application instance.
/// * `entity_content` - The entity representing the Content view to be updated.
///
/// # Returns
///
/// Returns a `gpui::Task<()>` representing the spawned background task.
fn start_background_subscriptions(
    app: &mut App,
    entity_content: &Entity<ContentView>,
) -> gpui::Task<()> {
    let entity_content = entity_content.clone();
    let (tx, mut rx) = channel::mpsc::channel::<Result<Vec<Item>, BackGroundError>>(10);

    // Keep a reference to the send channel so we can restart the background
    // if we lose connection.
    entity_content.update(app, |entity_view, _cx| {
        entity_view.article_sender.replace(tx.clone());
    });

    app.spawn(async move |app| {
        while let Some(items) = rx.next().await {
            match items {
                Ok(items) => {
                    let viewing_comments = entity_content
                        .read_with(app, |content: &ContentView, _app| !content.online);

                    if viewing_comments {
                        continue;
                    }

                    let current_ranking_map = items
                        .iter()
                        .enumerate()
                        .map(|(index, item)| (item.id, index))
                        .collect::<HashMap<_, _>>();

                    let current_comment_counts = items
                        .iter()
                        .map(|item| (item.id, item.descendants.unwrap_or(0)))
                        .collect::<HashMap<_, _>>();

                    // Create an ArticleView for each item.
                    let views = items
                        .into_iter()
                        .enumerate()
                        .map(|(index, article)| {
                            let order_change = app.read_entity(&entity_content, |content, _app| {
                                match content.article_ranks.get(&article.id) {
                                    Some(rank) => (*rank as i64) - (index as i64),
                                    None => 0,
                                }
                            });

                            let last_comment_count =
                                app.read_entity(&entity_content, |content, _app| {
                                    content.article_comment_counts.get(&article.id).cloned()
                                });

                            let background_refresh_count = app
                                .read_entity(&entity_content, |content, _app| {
                                    content.background_refresh_count
                                });

                            let comment_count_changed: i64 = if background_refresh_count > 0 {
                                let last_comment_count = last_comment_count.unwrap_or(0) as i64;
                                let current_comment_count = article.descendants.unwrap_or(0) as i64;

                                if last_comment_count > 0 && current_comment_count > 0 {
                                    current_comment_count - last_comment_count
                                } else {
                                    0
                                }
                            } else {
                                0
                            };

                            ArticleView::new(
                                app,
                                entity_content.clone(),
                                article,
                                order_change,
                                index + 1,
                                comment_count_changed,
                            )
                        })
                        .collect::<Vec<_>>();

                    app.update_entity(&entity_content, |content, cx| {
                        content.articles = views;
                        content.list_state.reset(content.articles.len());
                        content.article_ranks = current_ranking_map;
                        content.article_comment_counts = current_comment_counts;
                        content.background_refresh_count += 1;
                        cx.emit(ContentEvent::TotalArticles(content.articles.len()));
                        cx.emit(ContentEvent::TotalRefreshes(
                            content.background_refresh_count,
                        ));
                        cx.emit(ContentEvent::Error(None));
                        cx.notify();
                    });
                }
                Err(background_error) => {
                    error!("Received error from event source channel: {background_error:?}");
                    match background_error {
                        BackGroundError::Terminated(article_type) => {
                            app.update_entity(&entity_content, |_content_view, cx| {
                                cx.emit(ContentEvent::Terminated(article_type));
                                cx.notify();
                            })
                        }
                        BackGroundError::EventFailed(error) => {
                            app.update_entity(&entity_content, |_content_view, cx| {
                                cx.emit(ContentEvent::Error(Some(error)));
                                cx.notify();
                            })
                        }
                    };
                }
            }
        }
        log::warn!("Foreground events have terminated");
    })
    .detach();

    start_background_article_list_subscription(app, tx)
}

/// Starts a background subscription to the article list
/// and spawns a task to send articles to the foreground.
///
/// # Arguments
///
/// * `app` - A mutable reference to the application instance.
/// * `tx` - A sender channel for pushing article updates from background to foreground.
///
/// # Returns
///
/// Returns a `gpui::Task<()>` representing the spawned background task.
pub(crate) fn start_background_article_list_subscription(
    app: &mut App,
    mut tx: channel::mpsc::Sender<Result<Vec<Item>, BackGroundError>>,
) -> gpui::Task<()> {
    let ArticleSelection {
        viewing_article_type,
        viewing_article_total,
    } = app.read_global(|selection: &ArticleSelection, _app| *selection);

    info!("Starting background task for category {viewing_article_type} {viewing_article_total}");

    let client = app.read_global(|client: &ApiClientState, _app| client.0.clone());

    app.background_executor().spawn(Compat::new(async move {
        let (mut rx, handle) = subscribe_to_article_list(viewing_article_type);

        while let Some(event) = rx.recv().await {
            let article_ids = event
                .data
                .into_iter()
                .take(viewing_article_total)
                .collect::<Vec<_>>();

            let result = client
                .items(&article_ids)
                .try_collect::<Vec<_>>()
                .await
                .map_err(|err| {
                    BackGroundError::EventFailed(format!("Failed to fetch updated items: {err}"))
                });

            if let Err(err) = tx.send(result).await {
                error!("UI foreground send channel is closed: {err}");
                break;
            }
        }

        log::warn!("Background events have terminated");

        if let Err(err) = tx
            .send(Err(BackGroundError::Terminated(viewing_article_type)))
            .await
        {
            error!("Failed to send error {err}");
        }

        handle.abort();
    }))
}

impl Render for ContentView {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div().id("articles").overflow_scroll().p_1().m_1().children(
            self.articles
                .iter()
                .map(|article| div().m_1().child(article.clone())),
        )
    }
}
