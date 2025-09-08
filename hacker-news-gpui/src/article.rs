//! Article view.
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
    order_change_label: SharedString,
    order_change: i64,
}

impl ArticleView {
    pub fn new(app: &mut AsyncApp, item: Item, order_change: i64) -> anyhow::Result<Entity<Self>> {
        app.new(|_| Self {
            title: item.title.unwrap_or_default().into(),
            author: format!("by {}", item.by.clone()).into(),
            score: format!("{}", item.score,).into(),
            comments: format!("💬{}", item.kids.len()).into(),
            url: item.url.map(Into::into),
            order_change_label: if order_change == 0 {
                String::new()
            } else if order_change.is_positive() {
                format!("+{order_change}")
            } else {
                format!("{order_change}")
            }
            .into(),
            order_change,
        })
    }
}

impl Render for ArticleView {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let rank_change_col = div()
            .flex()
            .w(rems(1.5))
            .justify_end()
            .child(self.order_change_label.clone());

        let points_col = div()
            .flex()
            .w(rems(3.))
            .justify_end()
            .child(self.score.clone());

        let comments_col = div()
            .flex()
            .w(rems(3.))
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
                            .on_hover(move |hover, _window, app| {
                                if !hover {
                                    app.set_global::<UrlHover>(UrlHover(None));
                                } else if let Some(entity) = weak_entity.upgrade() {
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
            .text_size(px(15.0))
            .text_color(white())
            .when(self.order_change > 2, |div| div.text_color(rgb(0x66ff1a)))
            .when(self.order_change < -2, |div| div.text_color(rgb(0xff1a1a)))
            .w_full()
            .gap_x(px(5.0))
            .child(rank_change_col)
            .child(points_col)
            .child(comments_col)
            .child(title_col)
            .px_1()
            .border_color(rgb(0xEEEEEE))
    }
}
