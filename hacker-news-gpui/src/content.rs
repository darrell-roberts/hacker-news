//! Main content view
use std::rc::Rc;

use crate::{article::ArticleView, ApiClientState, AppState};
use anyhow::Context as _;
use async_compat::Compat;
use gpui::{
    div, list, prelude::*, px, App, AppContext, AsyncApp, Entity, EventEmitter, ListState,
    WeakEntity, Window,
};
use hacker_news_api::Item;

// Main content view.
pub struct Content {
    articles: Vec<Rc<Item>>,
    list_state: ListState,
}

pub struct TotalArticles(pub usize);

impl EventEmitter<TotalArticles> for Content {}

impl Content {
    /// Create a new content view.
    pub fn new(_cx: &mut Window, app: &mut App) -> Entity<Self> {
        app.new(|cx: &mut Context<Self>| {
            let list_state = ListState::new(0, gpui::ListAlignment::Top, px(5.0));
            Self::fetch_articles(cx);
            cx.observe_global::<AppState>(|view, cx| {
                println!("global observer on Entity<Content> fired");
                view.articles = Vec::new();
                Self::fetch_articles(cx)
            })
            .detach();
            Self {
                articles: Vec::new(),
                list_state,
            }
        })
    }

    fn fetch_articles(cx: &mut Context<Self>) {
        cx.spawn(async |view: WeakEntity<Content>, cx: &mut AsyncApp| {
            if let Err(err) = fetch_articles(view, cx).await {
                eprintln!("Failed to fetch articles: {err}");
            }
        })
        .detach();
    }
}

async fn fetch_articles(view: WeakEntity<Content>, cx: &mut AsyncApp) -> anyhow::Result<()> {
    let client = cx.read_global(|client: &ApiClientState, _app| client.0.clone())?;
    let view = view
        .upgrade()
        .context("Could not upgrade view weak reference")?;

    let (a_type, total) =
        cx.read_global(|r: &AppState, _cx| (r.viewing_article_type, r.viewing_article_total))?;

    // Run in compat since client uses tokio
    let new_articles = Compat::new(client.articles(total, a_type))
        .await
        .context("Failed to fetch")?;
    cx.update_entity(&view, move |view, cx| {
        view.articles = new_articles.into_iter().map(Rc::new).collect();
        view.list_state.reset(view.articles.len());
        cx.notify();
        cx.emit(TotalArticles(view.articles.len()));
    })
}

impl Render for Content {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let entity = cx.weak_entity();
        let articles = || {
            div().flex_grow().px_2().child(
                list(self.list_state.clone(), move |index, _window, app| {
                    if let Some(view) = entity.upgrade() {
                        let view = view.read(app);
                        match view.articles.get(index) {
                            Some(article) => {
                                ArticleView::new(app, article.clone()).into_any_element()
                            }
                            None => div().into_any(),
                        }
                    } else {
                        div().into_any()
                    }
                })
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
                articles()
            })
    }
}
