use crate::{
    content::{ContentEvent, ContentView},
    theme::Theme,
    ArticleSelection, UrlHover,
};
use chrono::Local;
use gpui::{
    div, prelude::*, rems, App, Entity, ParentElement, Render, SharedString, Styled, Window,
};

pub struct Footer {
    status_line: SharedString,
    url: Option<SharedString>,
    content: Entity<ContentView>,
    subscribed: bool,
}

impl Footer {
    pub fn new(_cx: &mut Window, app: &mut App, content: Entity<ContentView>) -> Entity<Self> {
        app.new(|cx| {
            cx.observe_global::<ArticleSelection>(move |footer: &mut Footer, cx| {
                footer.status_line = "Fetching...".into();
                cx.notify()
            })
            .detach();

            cx.subscribe(
                &content,
                |footer: &mut Footer, _content, event, _cx| match event {
                    ContentEvent::TotalArticles(n) => {
                        footer.status_line =
                            format!("Updated: {}, total {n}", Local::now().format("%D %T"),).into();
                    }
                    ContentEvent::ViewingComments(viewing) => {
                        footer.subscribed = !viewing;
                    }
                },
            )
            .detach();

            cx.observe_global::<UrlHover>(|footer: &mut Footer, cx| {
                footer.url = cx.global::<UrlHover>().0.clone();
                cx.notify();
            })
            .detach();

            Self {
                status_line: Default::default(),
                url: None,
                content,
                subscribed: true,
            }
        })
    }
}

impl Render for Footer {
    fn render(
        &mut self,
        window: &mut Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme: Theme = window.appearance().into();
        let content = self.content.clone();
        let subscribed = self.subscribed;

        div()
            .p_1()
            .bg(theme.surface())
            .text_size(rems(0.85))
            .child(self.url.clone().unwrap_or_default())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .child(self.status_line.clone())
                    .child(
                        div()
                            .id("resume")
                            .cursor_pointer()
                            .on_click(move |_event, _window, app| {
                                content.update(app, |_content: &mut ContentView, cx| {
                                    cx.emit(ContentEvent::ViewingComments(subscribed));
                                })
                            })
                            .when_else(!subscribed, |el| el.child("[*]"), |el| el.child("[~]")),
                    ),
            )
    }
}
