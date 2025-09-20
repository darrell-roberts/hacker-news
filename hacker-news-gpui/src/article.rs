//! Article view.
use crate::{theme::Theme, UrlHover};
use chrono::{DateTime, Utc};
use gpui::{
    div, img, prelude::*, px, rems, solid_background, AppContext, AsyncApp, Entity, Fill,
    FontWeight, Image, ImageSource, SharedString, StyleRefinement, Window,
};
use hacker_news_api::Item;
use std::sync::{Arc, LazyLock};

// An article view is rendered for each article item.
pub struct ArticleView {
    title: SharedString,
    author: SharedString,
    comments: Option<SharedString>,
    url: Option<SharedString>,
    order_change_label: SharedString,
    order_change: i64,
    age: SharedString,
    comment_image: ImageSource,
    id: u64,
}

/// An embedded SVG comment image.
static COMMENT_IMAGE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        include_bytes!("../assets/comment.svg").into(),
    ))
});

impl ArticleView {
    pub fn new(app: &mut AsyncApp, item: Item, order_change: i64) -> anyhow::Result<Entity<Self>> {
        app.new(|_| Self {
            id: item.id,
            title: item.title.unwrap_or_default().into(),
            author: format!("by {}", item.by.clone()).into(),
            comments: item
                .descendants
                .filter(|&n| n > 0)
                .map(|n| format!("{n}"))
                .map(Into::into),
            url: item.url.map(Into::into),
            order_change_label: if order_change == 0 {
                Default::default()
            } else {
                format!("{order_change}").into()
            },
            order_change,
            age: parse_date(item.time).unwrap_or_default().into(),
            comment_image: ImageSource::Image(Arc::clone(&COMMENT_IMAGE)),
        })
    }
}

impl Render for ArticleView {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme: Theme = window.appearance().into();

        let rank_change_col = div()
            .flex()
            .italic()
            .text_size(rems(0.75))
            .w(rems(0.75))
            .when(
                self.order_change.is_positive() && self.order_change > 0,
                |div| div.text_color(theme.text_increasing()),
            )
            .when(self.order_change.is_negative(), |div| {
                div.text_color(theme.text_decreasing())
            })
            .justify_end()
            .items_center()
            .child(self.order_change_label.clone());

        let id = self.id;
        let hover_element = |style: StyleRefinement| {
            style
                .font_weight(FontWeight::BOLD)
                .text_color(theme.text_color())
                .bg(Fill::Color(solid_background(theme.text_light_bar())))
        };

        let comments_col = div().w(rems(3.)).justify_end().id("comments").when_some(
            self.comments.as_ref(),
            |div, comments| {
                div.flex()
                    .cursor_pointer()
                    .rounded_md()
                    .on_click(move |_, _, app| {
                        app.open_url(&format!("https://news.ycombinator.com/item?id={id}"));
                    })
                    .hover(hover_element)
                    .flex()
                    .flex_row()
                    .child(comments.clone())
                    .child(img(self.comment_image.clone()))
            },
        );

        let url = self.url.clone();

        let weak_entity = cx.weak_entity();
        let title_col = div()
            .flex()
            .flex_row()
            .flex_grow()
            .child(
                div()
                    .rounded_md()
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
                    .hover(hover_element),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .italic()
                    .items_center()
                    .font_family(SharedString::new_static(".SystemUIFont"))
                    .text_size(px(12.0))
                    .child(self.author.clone())
                    .child(self.age.clone())
                    .gap_x(px(5.0)),
            )
            .gap_x(px(5.0));

        div()
            .flex()
            .flex_row()
            .font_family("Roboto, sans-serif")
            .text_size(px(15.0))
            .text_color(theme.text_color())
            .rounded_md()
            .bg(theme.bg())
            .when(self.order_change > 2, |div| {
                div.text_color(theme.text_increasing())
            })
            .when(self.order_change < -2, |div| {
                div.text_color(theme.text_decreasing())
            })
            .child(div().m_1().child(div().flex().flex_row().children([
                rank_change_col,
                div().child(comments_col),
                title_col,
            ])))
    }
}

/// Extract the duration from a UNIX time and convert duration into a human
/// friendly sentence.
fn parse_date(time: u64) -> Option<String> {
    let duration =
        DateTime::<Utc>::from_timestamp(time.try_into().ok()?, 0).map(|then| Utc::now() - then)?;

    let hours = duration.num_hours();
    let minutes = duration.num_minutes();
    let days = duration.num_days();

    match (days, hours, minutes) {
        (0, 0, 1) => "1 minute ago".to_string(),
        (0, 0, m) => format!("{m} minutes ago"),
        (0, 1, _) => "1 hour ago".to_string(),
        (0, h, _) => format!("{h} hours ago"),
        (1, _, _) => "1 day ago".to_string(),
        (d, _, _) => format!("{d} days ago"),
    }
    .into()
}
