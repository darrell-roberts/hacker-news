//! Main content view
use crate::{article::ArticleView, ApiClientState};
use async_compat::Compat;
use gpui::{div, list, prelude::*, px, AnyElement, ListState, View, ViewContext, WindowContext};
use hacker_news_api::{ArticleType, Item};

// Main content view.
pub struct Content {
    // articles: Vec<View<ArticleView>>,
    articles: Vec<Item>,
    list_state: ListState,
}

impl Content {
    /// Create a new content view.
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        let client = cx.global::<ApiClientState>().clone();

        let view = cx.new_view(|cx: &mut ViewContext<Self>| {
            let view = cx.view().downgrade();

            let list_state =
                ListState::new(0, gpui::ListAlignment::Top, px(5.0), move |idx, cx| {
                    if let Some(view) = view.upgrade() {
                        view.update(cx, |view, cx| view.render_article(idx, cx))
                    } else {
                        div().into_any()
                    }
                });

            Self {
                articles: Vec::new(),
                list_state,
            }
        });
        cx.spawn(|mut cx| {
            let view = view.clone();

            // Run in compat since client uses tokio
            Compat::new(async move {
                let new_articles = client.0.articles(75, ArticleType::Top).await.unwrap();
                // println!("fetched {} articles", new_articles.len());
                cx.update_view(&view, |view, cx| {
                    view.articles.extend(
                        new_articles.into_iter(), // .map(|item| ArticleView::new(cx, item)),
                    );
                    view.list_state.reset(view.articles.len());
                    cx.notify();
                })
                .unwrap();
            })
        })
        .detach();
        view
    }

    fn render_article(&self, ix: usize, cx: &mut ViewContext<Self>) -> AnyElement {
        match self.articles.get(ix) {
            Some(article) => ArticleView::new(cx, article.clone()).into_any(),
            None => div().into_any(),
        }
    }
}

impl Render for Content {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let articles = || {
            div()
                .flex_grow()
                .px_2()
                .child(list(self.list_state.clone()).size_full())
        };

        let loading = || div().child("Loading...");

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .child(if self.articles.is_empty() {
                loading()
            } else {
                articles()
            })
    }
}
