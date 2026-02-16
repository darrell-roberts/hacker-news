//! Main content view
use crate::{article::ArticleView, ApiClientState, ArticleSelection};
use async_compat::Compat;
use futures::{channel, SinkExt, StreamExt, TryStreamExt as _};
use gpui::{div, prelude::*, px, App, AppContext, Entity, EventEmitter, ListState, Window};
use hacker_news_api::{subscribe_to_article_list, Item};
use log::error;
use std::collections::HashMap;

// Main content view.
pub struct ContentView {
    articles: Vec<Entity<ArticleView>>,
    list_state: ListState,
    /// Tracks the ranking of each article so that if it moves up
    /// or down we can show by how much.
    article_ranks: HashMap<u64, usize>,
    /// Tracks the number of comments for an article so that when it
    /// changes we can show a visual indicator.
    article_comment_counts: HashMap<u64, u64>,
    pub stream_paused: bool,
    pub background_task: Option<gpui::Task<()>>,
    pub article_sender: Option<channel::mpsc::Sender<Vec<Item>>>,
}

pub enum ContentEvent {
    TotalArticles(usize),
    ViewingComments(bool),
}

impl EventEmitter<ContentEvent> for ContentView {}

impl ContentView {
    /// Create a new content view.
    pub fn new(_cx: &mut Window, app: &mut App) -> Entity<Self> {
        let entity_content = app.new(|cx: &mut Context<Self>| {
            cx.subscribe_self(|content, event, _cx| match event {
                ContentEvent::TotalArticles(_) => (),
                ContentEvent::ViewingComments(b) => {
                    content.stream_paused = *b;
                }
            })
            .detach();

            let list_state = ListState::new(0, gpui::ListAlignment::Top, px(5.0));

            Self {
                list_state,
                articles: Default::default(),
                article_ranks: Default::default(),
                stream_paused: false,
                background_task: None,
                article_sender: None,
                article_comment_counts: Default::default(),
            }
        });

        let background_task = start_background_subscriptions(app, &entity_content);
        entity_content.update(app, |content_view, _ctx| {
            content_view.background_task = Some(background_task);
        });
        entity_content
    }
}

/// Starts a background task that subscribes to the top stories stream,
/// fetches article data, and updates the Content entity accordingly.
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
/// This function is intended to be called once when initializing the Content view.
fn start_background_subscriptions(
    app: &mut App,
    entity_content: &Entity<ContentView>,
) -> gpui::Task<()> {
    let entity_content = entity_content.clone();
    let (tx, mut rx) = channel::mpsc::channel::<Vec<Item>>(10);

    entity_content.update(app, |entity_view, _cx| {
        entity_view.article_sender.replace(tx.clone());
    });

    app.spawn(async move |app| {
        while let Some(items) = rx.next().await {
            let updates_paused =
                entity_content.read_with(app, |content: &ContentView, _app| content.stream_paused);

            if updates_paused {
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

                    let last_comment_count = app.read_entity(&entity_content, |content, _app| {
                        content.article_comment_counts.get(&article.id).cloned()
                    });

                    let comment_count_changed = article.descendants != last_comment_count;

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
                cx.emit(ContentEvent::TotalArticles(content.articles.len()));
                cx.notify();
            });
        }
    })
    .detach();

    start_background_article_list_subscription(app, tx)
}

// Starts a background subscription to the article list
// and spawns a task to send articles to the foreground.
pub(crate) fn start_background_article_list_subscription(
    app: &mut App,
    mut tx: channel::mpsc::Sender<Vec<Item>>,
) -> gpui::Task<()> {
    let ArticleSelection {
        viewing_article_type,
        viewing_article_total,
    } = app.read_global(|selection: &ArticleSelection, _app| *selection);

    let client = app.read_global(|client: &ApiClientState, _app| client.0.clone());

    app.background_executor().spawn(Compat::new(async move {
        let (mut rx, handle) = subscribe_to_article_list(viewing_article_type);

        while let Some(event) = rx.recv().await {
            let article_ids = event
                .data
                .into_iter()
                .take(viewing_article_total)
                .collect::<Vec<_>>();
            // TODO: Would be good to see if we can compare the last article_ids with the
            // current so we only fetch items and send to the foreground when we have changes
            // in the subset.
            let articles = client.items(&article_ids).try_collect::<Vec<_>>().await;
            match articles {
                Ok(articles) => {
                    if let Err(err) = tx.send(articles).await {
                        error!("UI foreground send channel is closed: {err}");
                        break;
                    }
                }
                Err(err) => {
                    error!("Failed to collect updated items: {err}");
                }
            }
        }

        if let Err(err) = handle.await {
            error!("Subscription close failed {err}");
        };
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
