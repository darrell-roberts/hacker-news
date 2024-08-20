use gpui::{div, prelude::*};
use hacker_news_api::Item;

struct ArticleView {
    item: Item,
}

impl Render for ArticleView {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        div()
            .flex_row()
            .child(format!("{}", self.item.title.as_deref().unwrap_or("???")))
    }
}
