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

        // macOS draws native traffic lights over the (transparent) titlebar on the
        // left, so reserve space for them and skip our own window controls.
        let is_macos = cfg!(target_os = "macos");
        // In fullscreen the traffic lights are hidden, so the inset is no longer needed.
        let reserve_traffic_lights = is_macos && !window.is_fullscreen();

        div()
            .h(px(32.))
            .p_2()
            // Inset the title so it clears the native traffic lights on macOS.
            .when(reserve_traffic_lights, |this| this.pl(px(72.)))
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .id("title-bar-drag")
                    .flex_1()
                    .h_full()
                    // Vertically center the title so it sits on the same
                    // centerline as the native traffic lights.
                    .flex()
                    .items_center()
                    .on_mouse_down(MouseButton::Left, |event, window, _| {
                        if event.click_count > 1 {
                            window.zoom_window();
                        }
                        window.start_window_move();
                    })
                    .child("Hacker News Dashboard"),
            )
            .when(!is_macos, |this| {
                this.child(
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
            })
    }
}
