//! Header view.
use crate::AppState;
use gpui::{
    div, prelude::FluentBuilder, px, rgb, BorrowAppContext, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, Styled, Subscription, View, VisualContext, WindowContext,
};
use hacker_news_api::ArticleType;

/// Header view
pub struct Header {
    _state_subscription: Subscription,
}

impl Header {
    /// Create a new header view.
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self {
            _state_subscription: cx.observe_global::<AppState>(move |_state, cx| cx.notify()),
        })
    }
}

impl Render for Header {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let g = cx.global::<AppState>();

        let mk_article_type = |article_type: ArticleType| {
            div()
                .when(article_type == g.viewing_article_type, |div| {
                    div.text_bg(rgb(0x323232))
                        .border_1()
                        .text_color(rgb(0xFFFFFF))
                        .rounded(px(0.8))
                })
                .child(article_type.as_str().to_owned())
                .on_mouse_down(MouseButton::Left, move |_event, cx| {
                    cx.update_global::<AppState, _>(|state, _cx| {
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
                    div.text_bg(rgb(0x323232))
                        .border_1()
                        .text_color(rgb(0xFFFFFF))
                        .rounded(px(0.8))
                })
                .child(format!("{article_count}"))
                .on_mouse_down(MouseButton::Left, move |_event, cx| {
                    cx.update_global::<AppState, _>(|state, _cx| {
                        state.viewing_article_total = article_count;
                    })
                })
        });

        div()
            .flex()
            .flex_row()
            .font_family("Roboto, sans-serif")
            .text_size(px(24.0))
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
