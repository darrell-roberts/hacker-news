//! Header view.
use crate::ArticleSelection;
use gpui::{
    black, div, prelude::FluentBuilder, px, yellow, App, AppContext as _, BorrowAppContext, Entity,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement as _, Styled, Window,
};
use hacker_news_api::ArticleType;

/// Header view
pub struct Header {
    counts: [(usize, SharedString); 5],
}

impl Header {
    /// Create a new header view.
    pub fn new(_cx: &mut Window, app: &mut App) -> Entity<Self> {
        app.new(|_cx| Self {
            counts: [25, 50, 75, 100, 500].map(|n| (n, format!("{n}").into())),
        })
    }
}

impl Render for Header {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let article_selection = cx.global::<ArticleSelection>();

        let mk_article_type = |article_type: ArticleType| {
            div()
                .when(
                    article_type == article_selection.viewing_article_type,
                    |div| {
                        div.text_bg(yellow())
                            .border_1()
                            .text_color(black())
                            .rounded(px(0.8))
                    },
                )
                .child(
                    div()
                        .id(article_type.as_str())
                        .child(article_type.as_str())
                        .cursor_pointer()
                        .on_click(move |_event, _window, app| {
                            app.update_global(|state: &mut ArticleSelection, _cx| {
                                state.viewing_article_type = article_type;
                            });
                        }),
                )
        };

        let col1 = [ArticleType::Top, ArticleType::Best, ArticleType::New]
            .into_iter()
            .map(mk_article_type);

        let col2 = [ArticleType::Ask, ArticleType::Show, ArticleType::Job]
            .into_iter()
            .map(mk_article_type);

        let col3 = self
            .counts
            .clone()
            .into_iter()
            .map(|(article_count, label)| {
                div()
                    .when(
                        article_count == article_selection.viewing_article_total,
                        |div| {
                            div.text_bg(yellow())
                                .border_1()
                                .text_color(black())
                                .rounded(px(0.8))
                        },
                    )
                    .id(label.clone())
                    .child(label)
                    .cursor_pointer()
                    .on_click(move |_event, _window, app| {
                        app.update_global(|state: &mut ArticleSelection, _cx| {
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
            .justify_center()
            .children(col1)
            .child(div().border_4())
            .children(col2)
            .child(div().border_4())
            .children(col3)
            .px_1()
    }
}
