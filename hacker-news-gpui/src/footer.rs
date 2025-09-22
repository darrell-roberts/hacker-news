use crate::{content::Content, theme::Theme, ArticleSelection, UrlHover};
use chrono::Local;
use gpui::{
    div, prelude::*, rems, App, Entity, ParentElement, Render, SharedString, Styled, WeakEntity,
    Window,
};

pub struct Footer {
    status_line: SharedString,
    url: Option<SharedString>,
    content: WeakEntity<Content>,
}

impl Footer {
    pub fn new(_cx: &mut Window, app: &mut App, content: &Entity<Content>) -> Entity<Self> {
        let weak_content = content.downgrade();
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
                content: weak_content,
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

        let weak_content = self.content.clone();

        div()
            .p_1()
            .text_color(theme.text_color())
            .bg(theme.surface())
            .text_size(rems(0.75))
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
                                if let Some(content) = weak_content.upgrade() {
                                    content.update(app, |content: &mut Content, _app| {
                                        content.viewing_comment = false;
                                    })
                                }
                            })
                            .child("[~]"),
                    ),
            )
    }
}
