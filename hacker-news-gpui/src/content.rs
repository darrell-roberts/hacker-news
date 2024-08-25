//! Main content view
use crate::{article::ArticleView, ApiClientState, AppState};
use async_compat::Compat;
use gpui::{
    div, list, prelude::*, px, AnyElement, ListState, Subscription, View, ViewContext,
    WindowContext,
};
use hacker_news_api::Item;
use jiff::Zoned;

// Main content view.
pub struct Content {
    articles: Vec<Item>,
    list_state: ListState,
    _state_subscription: Subscription,
}

impl Content {
    /// Create a new content view.
    pub fn new(cx: &mut WindowContext) -> View<Self> {
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

            Self::fetch_articles(cx);

            Self {
                articles: Vec::new(),
                list_state,
                _state_subscription: cx.observe_global::<AppState>(|view, cx| {
                    view.articles = Vec::new();
                    Self::fetch_articles(cx)
                }),
            }
        });

        view
    }

    fn render_article(&self, ix: usize, cx: &mut ViewContext<Self>) -> AnyElement {
        match self.articles.get(ix) {
            Some(article) => ArticleView::new(cx, article.clone()).into_any(),
            None => div().into_any(),
        }
    }

    fn fetch_articles(cx: &mut ViewContext<Self>) {
        let client = cx.global::<ApiClientState>().clone();

        cx.spawn(|view, mut cx| {
            let view = view.upgrade().unwrap();

            // Run in compat since client uses tokio
            Compat::new(async move {
                let (a_type, total) = cx
                    .read_global::<AppState, _>(|r, _cx| {
                        (r.viewing_article_type, r.viewing_article_total)
                    })
                    .unwrap();

                let new_articles = client.articles(total, a_type).await.unwrap();
                cx.update_view(&view, |view, cx| {
                    view.articles = new_articles;
                    view.list_state.reset(view.articles.len());
                    cx.notify();
                })
                .unwrap();
            })
        })
        .detach();
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
            .size_full()
            .child(if self.articles.is_empty() {
                loading()
            } else {
                articles()
            })
    }
}
