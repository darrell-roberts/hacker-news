//! Main content view
use crate::{article::ArticleView, ApiClientState, ArticleSelection};
use async_compat::Compat;
use futures::{channel, SinkExt, StreamExt, TryStreamExt as _};
use gpui::{div, prelude::*, px, App, AppContext, Entity, EventEmitter, ListState, Window};
use hacker_news_api::{subscribe_top_stories, Item};
use log::error;
use std::collections::HashMap;

// Main content view.
pub struct Content {
    articles: Vec<Entity<ArticleView>>,
    list_state: ListState,
    article_ranks: HashMap<u64, usize>,
    pub viewing_comment: bool,
}

pub struct TotalArticles(pub usize);

impl EventEmitter<TotalArticles> for Content {}

impl Content {
    /// Create a new content view.
    pub fn new(_cx: &mut Window, app: &mut App) -> Entity<Self> {
        let entity = app.new(|_cx: &mut Context<Self>| {
            let list_state = ListState::new(0, gpui::ListAlignment::Top, px(5.0));

            Self {
                articles: Default::default(),
                list_state,
                article_ranks: Default::default(),
                viewing_comment: false,
            }
        });

        let weak_entity = entity.downgrade();

        let (mut tx, mut rx) = channel::mpsc::channel::<Vec<Item>>(10);

        app.spawn(async move |app| {
            while let Some(items) = rx.next().await {
                if let Some(entity) = weak_entity.upgrade() {
                    let viewing_comment = entity
                        .read_with(app, |content: &Content, _app| content.viewing_comment)
                        .unwrap_or_default();

                    if viewing_comment {
                        continue;
                    }

                    let ranking_map = items
                        .iter()
                        .enumerate()
                        .map(|(index, item)| (item.id, index))
                        .collect::<HashMap<_, _>>();

                    let views = items
                        .into_iter()
                        .enumerate()
                        .map(|(index, article)| {
                            let order_change = app
                                .read_entity(&entity, |content, _app| {
                                    match content.article_ranks.get(&article.id) {
                                        Some(rank) => (*rank as i64) - (index as i64),
                                        None => 0,
                                    }
                                })
                                .unwrap();
                            ArticleView::new(
                                app,
                                weak_entity.clone(),
                                article,
                                order_change,
                                index + 1,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>();

                    match views {
                        Ok(views) => {
                            let result = app.update_entity(&entity, |content, cx| {
                                content.articles = views;
                                content.list_state.reset(content.articles.len());
                                content.article_ranks = ranking_map;
                                cx.emit(TotalArticles(content.articles.len()));
                                cx.notify();
                            });
                            if let Err(err) = result {
                                error!("Failed to updated articles: {err}");
                            }
                        }
                        Err(err) => {
                            error!("Could not create article view. App shutting down? {err}");
                        }
                    }
                }
            }
        })
        .detach();

        let view_total =
            app.read_global(|selection: &ArticleSelection, _app| selection.viewing_article_total);

        let client = app.read_global(|client: &ApiClientState, _app| client.0.clone());

        app.background_executor()
            .spawn(Compat::new(async move {
                let (mut rx, handle) = subscribe_top_stories();

                while let Some(event) = rx.recv().await {
                    let article_ids = event.data.into_iter().take(view_total).collect::<Vec<_>>();
                    let articles = client.items(&article_ids).try_collect::<Vec<_>>().await;
                    match articles {
                        Ok(articles) => {
                            tx.send(articles).await.unwrap();
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
            .detach();

        entity
    }
}

impl Render for Content {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div().id("articles").overflow_scroll().p_1().m_1().children(
            self.articles
                .iter()
                .map(|article| div().m_1().child(article.clone())),
        )
    }
}
