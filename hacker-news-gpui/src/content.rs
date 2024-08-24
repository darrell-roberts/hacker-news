//! Main content view
use crate::{article::ArticleView, ApiClientState};
use async_compat::Compat;
use gpui::{div, prelude::*, rgb, View, WindowContext};
use hacker_news_api::ArticleType;

// Main content view.
pub struct Content {
    articles: Vec<View<ArticleView>>,
}

impl Content {
    /// Create a new content view.
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        let client = cx.global::<ApiClientState>().clone();

        let view = cx.new_view(|_| Self {
            articles: Vec::new(),
        });
        cx.spawn(|mut cx| {
            let view = view.clone();

            // Run in compat since client uses tokio
            Compat::new(async move {
                let new_articles = client.0.articles(10, ArticleType::Top).await.unwrap();
                println!("fetched {} articles", new_articles.len());
                cx.update_view(&view, |view, cx| {
                    view.articles.extend(
                        new_articles
                            .into_iter()
                            .map(|item| ArticleView::new(cx, item)),
                    );
                })
                .unwrap();
            })
        })
        .detach();
        view
    }
}

impl Render for Content {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0xffffff))
            // .child("Articles")
            .children(self.articles.clone())
    }
}
