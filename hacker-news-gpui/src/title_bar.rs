//! Title bar.
use crate::{common::hover_element, theme::Theme};
use gpui::{App, Entity, MouseButton, Render, StyleRefinement, Window, div, prelude::*, px};

pub struct TitleBar;

impl TitleBar {
    /// Create a new titlebar with title, minimize, maximize and close controls.
    pub fn new(_window: &mut Window, app: &mut App) -> Entity<Self> {
        app.new(|_cx| Self {})
    }
}

impl Render for TitleBar {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme: Theme = window.appearance().into();

        let button_hover = |style: StyleRefinement| hover_element(theme)(style.cursor_pointer());

        div()
            .h(px(32.))
            .p_2()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .id("title-bar-drag")
                    .flex_1()
                    .h_full()
                    .on_mouse_down(MouseButton::Left, |event, window, _| {
                        if event.click_count > 1 {
                            window.zoom_window();
                        }
                        window.start_window_move();
                    })
                    .child("Hacker News Dashboard"),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        /* minimize button */
                        div()
                            .child("−")
                            .id("min")
                            .flex_1()
                            .p_1()
                            .hover(button_hover)
                            .on_click(|_, window, _cx| {
                                window.minimize_window();
                            }),
                    )
                    .child(
                        /* maximize button */
                        div()
                            .child(if window.is_maximized() { "❐" } else { "□" })
                            .id("max")
                            .p_1()
                            .hover(button_hover)
                            .on_click(|_, window, _| {
                                window.zoom_window();
                            }),
                    )
                    .child(
                        /* close button */
                        div()
                            .child("✕")
                            .id("close")
                            .p_1()
                            .hover(button_hover)
                            .on_click(|_, _, cx| cx.quit()),
                    ),
            )
    }
}
