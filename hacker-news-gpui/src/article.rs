//! Article view.
use gpui::{div, prelude::*, px, rgb, Font, FontStyle, Style, View, WindowContext};
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
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let title = self.item.title.clone().unwrap_or_default();

        let comments_col = div()
            .flex()
            .w(px(50.0))
            .justify_center()
            .child(format!("{}", self.item.kids.len()));

        let author = div()
            .italic()
            .text_size(px(14.0))
            .justify_end()
            .child(format!("by {}", self.item.by.clone()));

        let title_col = div()
            .flex()
            .flex_row()
            .flex_grow()
            .child(title)
            .child(author)
            .gap_x(px(5.0));

        div()
            .flex()
            .flex_row()
            .font_family("Roboto, sans-serif")
            .text_size(px(18.0))
            .w_full()
            .gap_x(px(5.0))
            .child(comments_col)
            .child(title_col)
            .px_1()
            .border_color(rgb(0xEEEEEE))
    }
}
