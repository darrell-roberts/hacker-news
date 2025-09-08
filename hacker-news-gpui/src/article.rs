//! Article view.
use std::cmp::Ordering;

use crate::UrlHover;
use gpui::{
    black, div, prelude::*, px, rems, rgb, solid_background, white, AppContext, AsyncApp, Entity,
    Fill, FontWeight, SharedString, Window,
};
use hacker_news_api::Item;

// An article view is rendered for each article item.
pub struct ArticleView {
    title: SharedString,
    author: SharedString,
    score: SharedString,
    comments: SharedString,
    url: Option<SharedString>,
}

impl ArticleView {
    pub fn new(app: &mut AsyncApp, item: Item, order_change: Ordering) -> Entity<Self> {
        app.new(|_| Self {
            title: item.title.unwrap_or_default().into(),
            author: format!("by {}", item.by.clone()).into(),
            score: format!(
                "{:2}{:>5}",
                match order_change {
                    Ordering::Less => "-",
                    Ordering::Equal => "",
                    Ordering::Greater => "+",
                },
                item.score,
            )
            .into(),
            comments: format!("💬{}", item.kids.len()).into(),
            url: item.url.map(Into::into),
        })
        .unwrap()
    }
}

impl Render for ArticleView {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let points_col = div()
            .flex()
            .w(rems(4.5))
            .justify_start()
            .child(self.score.clone());

        let comments_col = div()
            .flex()
            .w(rems(4.))
            .justify_start()
            .child(self.comments.clone());

        let author = div()
            .italic()
            .font_family(SharedString::new_static(".SystemUIFont"))
            .text_size(px(12.0))
            .justify_end()
            .child(self.author.clone());

        let url = self.url.clone();

        let weak_entity = cx.weak_entity();
        let title_col = div()
            .flex()
            .flex_row()
            .flex_grow()
            .child(
                div()
                    .child(
                        div()
                            .id("title")
                            .child(self.title.clone())
                            .cursor_pointer()
                            .on_click(move |_, _, app| {
                                if let Some(url) = url.as_deref() {
                                    app.open_url(url.as_ref());
                                }
                            })
                            .on_hover(move |_hover, _window, app| {
                                if let Some(entity) = weak_entity.upgrade() {
                                    let view = entity.read(app);
                                    app.set_global::<UrlHover>(UrlHover(view.url.clone()));
                                }
                            }),
                    )
                    .hover(|style| {
                        style
                            .font_weight(FontWeight::BOLD)
                            .text_color(black())
                            .bg(Fill::Color(solid_background(rgb(0x00134d))))
                    }),
            )
            .child(author)
            .gap_x(px(5.0));

        div()
            .flex()
            .flex_row()
            .font_family("Roboto, sans-serif")
            .text_size(px(14.0))
            .text_color(white())
            .w_full()
            .gap_x(px(5.0))
            .child(points_col)
            .child(comments_col)
            .child(title_col)
            .px_1()
            .border_color(rgb(0xEEEEEE))
    }
}
