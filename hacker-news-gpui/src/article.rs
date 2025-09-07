//! Article view.

use gpui::{
    div, prelude::*, px, rems, rgb, white, App, Entity, InteractiveText, SharedString, StyledText,
    Window,
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
    pub fn new(app: &mut App, item: &Item) -> Entity<Self> {
        app.new(|_| Self {
            title: item.title.clone().unwrap_or_default().into(),
            author: format!("by {}", item.by.clone()).into(),
            score: format!("🔼{}", item.score).into(),
            comments: format!("💬{}", item.kids.len()).into(),
            url: item.url.as_ref().map(Into::into),
        })
    }
}

impl Render for ArticleView {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut gpui::Context<Self>,
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
            // .text_size(px(14.0))
            .justify_end()
            .child(self.author.clone());

        let url = self.url.clone();
        let title_col = div()
            .flex()
            .flex_row()
            .flex_grow()
            .child(
                InteractiveText::new("title", StyledText::new(self.title.clone())).on_click(
                    vec![0..self.title.len()],
                    move |_index, _window, app| {
                        println!("Title clicked");
                        if let Some(url) = url.as_deref() {
                            println!("Opening url {url:?}");
                            app.open_url(url.as_ref());
                        }
                    },
                ),
            )
            // .child(div().child(self.title.clone()).on_mouse_down(
            //     MouseButton::Left,
            //     cx.listener(move |_view, _event, _window, cx| {
            //         println!("Title clicked");
            //         if let Some(url) = url.as_deref() {
            //             println!("Opening url {url:?}");
            //             cx.open_url(url.as_ref());
            //         }
            //     }),
            // ))
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
