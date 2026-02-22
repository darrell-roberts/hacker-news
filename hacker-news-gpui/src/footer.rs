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

/// Footer view and state.
pub struct FooterView {
    /// The current status line displayed in the footer.
    status_line: SharedString,
    /// The currently hovered URL, if any.
    url: Option<SharedString>,
    /// Reference to the ContentView entity.
    content: Entity<ContentView>,
    /// Whether the stream is subscribed (online) or paused.
    subscribed: bool,
    /// The total number of refreshes resulting from a server side event, as a string.
    total_refreshes: SharedString,
    /// Any error message to display, if present.
    error: Option<SharedString>,
}

impl FooterView {
    /// Create a new footer view entity.
    ///
    ///
    /// # Arguments
    ///
    /// * `_cx` - A mutable reference to the Window context.
    /// * `app` - A mutable reference to the App.
    /// * `content_entity` - An Entity representing the ContentView.
    ///
    /// # Returns
    ///
    /// Returns an Entity of FooterView.
    pub fn new(
        _cx: &mut Window,
        app: &mut App,
        content_entity: Entity<ContentView>,
    ) -> Entity<Self> {
        app.new(|cx| {
            cx.observe_global::<ArticleSelection>(move |footer: &mut FooterView, cx| {
                footer.status_line = "Fetching...".into();
                cx.notify()
            })
            .detach();

            cx.subscribe(
                &content_entity,
                |footer: &mut FooterView, _content_entity, event, _cx| match event {
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
                content: content_entity,
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
        let content_entity = self.content.clone();
        let subscribed = self.subscribed;

        div()
            .p_1()
            .bg(theme.surface())
            .text_size(rems(0.85))
            .when_some(self.error.as_ref(), |div, error| {
                div.child(
                    gpui::div()
                        .text_color(gpui::red())
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(error.clone()),
                )
            })
            .child(self.url.clone().unwrap_or_default())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .child(self.status_line.clone())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .child(
                                div()
                                    .id("resume")
                                    .cursor_pointer()
                                    .tooltip({
                                        let content_entity = content_entity.clone();
                                        move |window, app| {
                                            Tooltip::new(window, app, content_entity.clone()).into()
                                        }
                                    })
                                    .on_click(move |_event, _window, app| {
                                        content_entity.update(
                                            app,
                                            |_content: &mut ContentView, cx| {
                                                cx.emit(ContentEvent::ViewingComments(subscribed));
                                            },
                                        )
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

/// A simple tooltip
struct Tooltip {
    content_entity: Entity<ContentView>,
}

impl Tooltip {
    /// Create a new tooltip entity.
    ///
    /// # Arguments
    ///
    /// * `_window` - A mutable reference to the Window.
    /// * `cx` - A mutable reference to the App.
    /// * `content` - An Entity representing the ContentView.
    ///
    /// # Returns
    ///
    /// Returns an Entity of Tooltip.
    fn new(
        _window: &mut Window,
        cx: &mut App,
        content_entity: Entity<ContentView>,
    ) -> Entity<Self> {
        cx.new(|_cx| Self { content_entity })
    }
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let stream_paused = self.content_entity.read(cx).stream_paused;
        div()
            .bg(black())
            .text_color(white())
            .rounded(rems(0.75))
            .p_1()
            .child(if stream_paused { "Resume" } else { "Pause" })
    }
}
