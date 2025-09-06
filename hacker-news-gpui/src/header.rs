//! Header view.
use crate::AppState;
use gpui::{
    black, div, prelude::FluentBuilder, px, yellow, App, AppContext as _, BorrowAppContext, Entity,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Render, Styled, Window,
};
use hacker_news_api::ArticleType;

/// Header view
pub struct Header;

impl Header {
    /// Create a new header view.
    pub fn new(_cx: &mut Window, app: &mut App) -> Entity<Self> {
        app.new(|cx| {
            cx.observe_global::<AppState>(move |_state, cx| cx.notify())
                .detach();
            Self {}
        })
    }
}

impl Render for Header {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let g = cx.global::<AppState>();

        let mk_article_type = |article_type: ArticleType| {
            div()
                .when(article_type == g.viewing_article_type, |div| {
                    div.text_bg(yellow())
                        .border_1()
                        .text_color(black())
                        .rounded(px(0.8))
                })
                .child(article_type.as_str().to_owned())
                .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                    cx.update_global(|state: &mut AppState, _cx| {
                        state.viewing_article_type = article_type;
                    });
                })
        };

        let col1 = [ArticleType::Top, ArticleType::Best, ArticleType::New]
            .into_iter()
            .map(mk_article_type);

        let col2 = [ArticleType::Ask, ArticleType::Show, ArticleType::Job]
            .into_iter()
            .map(mk_article_type);

        let col3 = [25, 50, 75, 100, 500].into_iter().map(|article_count| {
            div()
                .when(article_count == g.viewing_article_total, |div| {
                    div.text_bg(yellow())
                        .border_1()
                        .text_color(black())
                        .rounded(px(0.8))
                })
                .child(format!("{article_count}"))
                .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                    cx.update_global(|state: &mut AppState, _cx| {
                        state.viewing_article_total = article_count;
                    })
                })
        });

        div()
            .flex()
            .flex_row()
            .font_family("Roboto, sans-serif")
            .text_size(px(24.0))
            .text_color(yellow())
            .gap_x(px(10.0))
            .w_full()
            .children(col1)
            .child(div().border_4())
            .children(col2)
            .child(div().border_4())
            .children(col3)
            .px_1()
    }
}
