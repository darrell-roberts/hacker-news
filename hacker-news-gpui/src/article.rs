//! Article view.
use std::rc::Rc;

use gpui::{div, prelude::*, px, rems, rgb, white, App, Entity, MouseButton, Window};
use hacker_news_api::Item;

// An article view is rendered for each article item.
pub struct ArticleView {
    item: Rc<Item>,
}

impl ArticleView {
    pub fn new(app: &mut App, item: Rc<Item>) -> Entity<Self> {
        app.new(|_| Self { item })
    }
}

impl Render for ArticleView {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let title = self.item.title.clone().unwrap_or_default();

        let points_col = div()
            .flex()
            .w(rems(4.5))
            .justify_start()
            .child(format!("🔼{}", self.item.score));

        let comments_col = div()
            .flex()
            .w(rems(4.))
            .justify_start()
            .child(format!("💬{}", self.item.kids.len()));

        let author = div()
            .italic()
            .text_size(px(14.0))
            .justify_end()
            .child(format!("by {}", self.item.by.clone()));

        let title_col = div()
            .flex()
            .flex_row()
            .flex_grow()
            .child(div().child(title).on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _event, _window, cx| {
                    println!("Title clicked");
                    if let Some(url) = view.item.url.as_deref() {
                        println!("Opening url {url}");
                        cx.open_url(url);
                    }
                }),
            ))
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
