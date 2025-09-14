use crate::{content::Content, ArticleSelection, UrlHover};
use chrono::Local;
use gpui::{
    div, prelude::*, rems, rgb, App, Entity, ParentElement, Render, SharedString, Styled, Window,
};

pub struct Footer {
    status_line: SharedString,
    url: Option<SharedString>,
}

impl Footer {
    pub fn new(_cx: &mut Window, app: &mut App, content: &Entity<Content>) -> Entity<Self> {
        app.new(|cx| {
            cx.observe_global::<ArticleSelection>(move |footer: &mut Footer, cx| {
                footer.status_line = "Fetching...".into();
                cx.notify()
            })
            .detach();

            cx.subscribe(
                content,
                |footer: &mut Footer, _content, total_articles, _cx| {
                    footer.status_line = format!(
                        "Updated: {}, total {}",
                        Local::now().format("%D %T"),
                        total_articles.0
                    )
                    .into();
                },
            )
            .detach();

            cx.observe_global::<UrlHover>(|footer: &mut Footer, cx| {
                footer.url = cx.global::<UrlHover>().0.clone();
            })
            .detach();

            Self {
                status_line: Default::default(),
                url: None,
            }
        })
    }
}

impl Render for Footer {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            // .text_color(rgb(0x424242))
            .text_color(rgb(0xeeeeee))
            .text_size(rems(0.75))
            // .h(px(55.))
            .when_some(self.url.as_ref(), |div, url| div.child(url.clone()))
            .child(self.status_line.clone())
    }
}
