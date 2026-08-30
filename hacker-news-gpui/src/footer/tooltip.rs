//! Tooltip view for footer view.
use crate::content::ContentView;
use gpui::{
    App, Entity, ParentElement, Render, Styled, Window, black, div, prelude::*, rems, white,
};

/// A simple tooltip
pub(super) struct Tooltip {
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
    pub(super) fn new(
        _window: &mut Window,
        cx: &mut App,
        content_entity: Entity<ContentView>,
    ) -> Entity<Self> {
        cx.new(|_cx| Self { content_entity })
    }
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let online = self.content_entity.read(cx).online;
        div()
            .bg(black())
            .opacity(0.70)
            .text_color(white())
            .rounded(rems(0.75))
            .p_1()
            .child(if online { "Pause" } else { "Resume" })
    }
}
