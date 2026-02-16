//! The footer view.
use crate::{
    content::{ContentEvent, ContentView},
    theme::Theme,
    ArticleSelection, UrlHover,
};
use chrono::Local;
use gpui::{
    black, div, prelude::*, rems, white, App, Entity, ParentElement, Render, SharedString, Styled,
    Window,
};

pub struct FooterView {
    status_line: SharedString,
    url: Option<SharedString>,
    content: Entity<ContentView>,
    subscribed: bool,
    total_refreshes: SharedString,
    error: Option<SharedString>,
}

impl FooterView {
    pub fn new(_cx: &mut Window, app: &mut App, content: Entity<ContentView>) -> Entity<Self> {
        app.new(|cx| {
            cx.observe_global::<ArticleSelection>(move |footer: &mut FooterView, cx| {
                footer.status_line = "Fetching...".into();
                cx.notify()
            })
            .detach();

            cx.subscribe(
                &content,
                |footer: &mut FooterView, _content, event, _cx| match event {
                    ContentEvent::TotalArticles(n) => {
                        footer.status_line =
                            format!("Updated: {}, total {n}", Local::now().format("%D %T"),).into();
                    }
                    ContentEvent::ViewingComments(viewing) => {
                        footer.subscribed = !viewing;
                    }
                    ContentEvent::TotalRefreshes(refresh_count) => {
                        footer.total_refreshes = format!("Refresh count: {refresh_count}").into();
                    }
                    ContentEvent::Error(error) => {
                        footer.error = error.as_ref().map(Into::into);
                    }
                },
            )
            .detach();

            cx.observe_global::<UrlHover>(|footer: &mut FooterView, cx| {
                footer.url = cx.global::<UrlHover>().0.clone();
                cx.notify();
            })
            .detach();

            Self {
                status_line: Default::default(),
                url: None,
                content,
                subscribed: true,
                total_refreshes: Default::default(),
                error: None,
            }
        })
    }
}

impl Render for FooterView {
    fn render(
        &mut self,
        window: &mut Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme: Theme = window.appearance().into();
        let content = self.content.clone();
        let content_tooltip = content.clone();
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
                    .when_some(self.error.as_ref(), |div, error| {
                        div.child(
                            gpui::div()
                                .text_color(gpui::red())
                                .font_weight(gpui::FontWeight::BOLD)
                                .child(error.clone()),
                        )
                    })
                    .child(self.status_line.clone())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .child(
                                div()
                                    .id("resume")
                                    .cursor_pointer()
                                    .tooltip(move |window, app| {
                                        Tooltip::new(window, app, content_tooltip.clone()).into()
                                    })
                                    .on_click(move |_event, _window, app| {
                                        content.update(app, |_content: &mut ContentView, cx| {
                                            cx.emit(ContentEvent::ViewingComments(subscribed));
                                        })
                                    })
                                    .when_else(
                                        subscribed,
                                        |el| el.child("[online]"),
                                        |el| el.child("[paused]"),
                                    ),
                            )
                            .child(self.total_refreshes.clone()),
                    ),
            )
    }
}

struct Tooltip {
    content: Entity<ContentView>,
}

impl Tooltip {
    fn new(_window: &mut Window, cx: &mut App, content: Entity<ContentView>) -> Entity<Self> {
        cx.new(|_cx| Self { content })
    }
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content_view = self.content.read(cx);
        div()
            .bg(black())
            .text_color(white())
            .rounded(rems(0.75))
            .p_1()
            .child(if content_view.stream_paused {
                "Resume"
            } else {
                "Pause"
            })
    }
}
