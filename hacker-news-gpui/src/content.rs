//! Main content view

use crate::{article::ArticleView, ApiClientState};
use async_compat::Compat;
use futures::{channel, SinkExt, StreamExt, TryStreamExt as _};
use gpui::{div, list, prelude::*, px, App, AppContext, Entity, EventEmitter, ListState, Window};
use hacker_news_api::{subscribe_top_stories, Item};

// Main content view.
pub struct Content {
    articles: Vec<Entity<ArticleView>>,
    list_state: ListState,
}

pub struct TotalArticles(pub usize);

impl EventEmitter<TotalArticles> for Content {}

impl Content {
    /// Create a new content view.
    pub fn new(_cx: &mut Window, app: &mut App) -> Entity<Self> {
        let entity = app.new(|_cx: &mut Context<Self>| {
            let list_state = ListState::new(0, gpui::ListAlignment::Top, px(5.0));
            // Self::fetch_articles(cx);

            // When we change the type or number of articles in the header
            // we will fetch with the new viewing options.
            // cx.observe_global::<ArticleSelection>(|view, cx| {
            //     view.articles = Default::default();
            //     Self::fetch_articles(cx)
            // })
            // .detach();

            Self {
                articles: Default::default(),
                list_state,
            }
        });

        let weak_entity = entity.downgrade();
        let client = app.read_global(|client: &ApiClientState, _app| client.0.clone());

        let (mut tx, mut rx) = channel::mpsc::channel::<Vec<Item>>(100);

        app.spawn(async move |app| {
            while let Some(items) = rx.next().await {
                if let Some(entity) = weak_entity.upgrade() {
                    let views = items
                        .into_iter()
                        .map(|article| ArticleView::new(app, article))
                        .collect();
                    let result = app.update_entity(&entity, |content, cx| {
                        content.articles = views;
                        content.list_state.reset(content.articles.len());
                        cx.emit(TotalArticles(content.articles.len()));
                        cx.notify();
                    });
                    if let Err(err) = result {
                        eprintln!("Failed to updated articles: {err}");
                    }
                }
            }
        })
        .detach();

        app.background_executor()
            .spawn(Compat::new(async move {
                let (mut rx, handle) = subscribe_top_stories();

                while let Some(event) = rx.recv().await {
                    let top_50 = event.data.into_iter().take(50).collect::<Vec<_>>();
                    let articles = client.items(&top_50).try_collect::<Vec<_>>().await;
                    match articles {
                        Ok(articles) => {
                            tx.send(articles).await.unwrap();
                        }
                        Err(err) => {
                            eprintln!("Failed to collect updated items: {err}");
                        }
                    }
                }

                if let Err(err) = handle.await {
                    eprintln!("Subscription close failed {err}");
                };
            }))
            .detach();

        entity
    }

    // fn fetch_articles(cx: &mut Context<Self>) {
    //     cx.spawn(async |view: WeakEntity<Content>, cx: &mut AsyncApp| {
    //         if let Err(err) = fetch_articles(view, cx).await {
    //             eprintln!("Failed to fetch articles: {err}");
    //         }
    //     })
    //     .detach();
    // }
}

// async fn fetch_articles(view: WeakEntity<Content>, cx: &mut AsyncApp) -> anyhow::Result<()> {
//     let client = cx.read_global(|client: &ApiClientState, _app| client.0.clone())?;
//     let view = view
//         .upgrade()
//         .context("Could not upgrade view weak reference")?;

//     let (article_type, total) = cx.read_global(|r: &ArticleSelection, _cx| {
//         (r.viewing_article_type, r.viewing_article_total)
//     })?;

//     // Run in compat since my client uses tokio
//     let new_articles = Compat::new(client.articles(total, article_type))
//         .await
//         .context("Failed to fetch")?;

//     cx.update_entity(&view, move |view, cx| {
//         view.articles = new_articles
//             .into_iter()
//             .map(|article| ArticleView::new(cx, article))
//             .collect();

//         view.list_state.reset(view.articles.len());
//         cx.notify();
//         cx.emit(TotalArticles(view.articles.len()));
//     })
// }

impl Render for Content {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let entity = cx.weak_entity();
        let render_articles = || {
            div().flex_grow().px_2().child(
                list(
                    self.list_state.clone(),
                    move |index, _window, app| match entity.upgrade() {
                        Some(content) => {
                            let view = content.read(app);
                            let articles = view.articles.clone();
                            articles[index].clone().into_any_element()
                        }
                        None => div().into_any(),
                    },
                )
                .size_full(),
            )
        };

        let loading = || div().child("Loading...");

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(if self.articles.is_empty() {
                loading()
            } else {
                render_articles()
            })
    }
}
