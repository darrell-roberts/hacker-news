use crate::{content::Content, AppState, UrlHover};
use gpui::{
    div, prelude::FluentBuilder, px, white, App, AppContext as _, Entity, ParentElement, Render,
    SharedString, Styled, Window,
};
use jiff::Zoned;

pub struct Footer {
    status_line: SharedString,
    url: Option<SharedString>,
}

impl Footer {
    pub fn new(_cx: &mut Window, app: &mut App, content: &Entity<Content>) -> Entity<Self> {
        app.new(|cx| {
            cx.observe_global::<AppState>(move |footer: &mut Footer, cx| {
                println!("updating status line");
                footer.status_line = "Fetching...".into();
                cx.notify()
            })
            .detach();

            cx.subscribe(
                content,
                |footer: &mut Footer, _content, total_articles, _cx| {
                    footer.status_line = format!(
                        "Updated: {}, total {}",
                        Zoned::now().strftime("%D %T"),
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
            .text_color(white())
            .h(px(50.))
            .child(self.status_line.clone())
            .when_some(self.url.as_ref(), |div, url| div.child(url.clone()))
    }
}
