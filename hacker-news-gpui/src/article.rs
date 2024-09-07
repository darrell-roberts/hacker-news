//! Article view.
use gpui::{div, prelude::*, px, rems, rgb, MouseButton, View, WindowContext};
use hacker_news_api::Item;

// An article view is rendered for each article item.
pub struct ArticleView {
    item: Item,
}

impl ArticleView {
    pub fn new(cx: &mut WindowContext, item: Item) -> View<Self> {
        cx.new_view(|_| Self { item })
    }
}

impl Render for ArticleView {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
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
                cx.listener(|view, _event, cx| {
                    if let Some(url) = view.item.url.as_deref() {
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
            .w_full()
            .gap_x(px(5.0))
            .child(points_col)
            .child(comments_col)
            .child(title_col)
            .px_1()
            .border_color(rgb(0xEEEEEE))
    }
}
