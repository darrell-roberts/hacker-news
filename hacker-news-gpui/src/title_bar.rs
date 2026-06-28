//! Title bar.
use crate::{common::hover_element, theme::Theme};
use gpui::{
    App, Div, Entity, MouseButton, OwnedMenu, Render, SharedString, StyleRefinement, Window,
    anchored, deferred, div, prelude::*, px,
};

pub struct TitleBar {
    opened_menu: Option<SharedString>,
}

impl TitleBar {
    /// Create a new titlebar with title, minimize, maximize and close controls.
    pub fn new(_window: &mut Window, app: &mut App) -> Entity<Self> {
        app.new(|_cx| Self { opened_menu: None })
    }

    fn render_menus(&self, cx: &gpui::Context<Self>, theme: Theme) -> impl gpui::IntoElement {
        match cx.get_menus() {
            Some(menus) => div()
                .flex()
                .flex_row()
                .gap_2()
                .children(menus.iter().map(|menu| self.menu_button(cx, theme, menu))),
            None => div(),
        }
    }

    fn menu_button(
        &self,
        cx: &gpui::Context<Self>,
        theme: Theme,
        menu: &OwnedMenu,
    ) -> impl IntoElement {
        let menu_name = menu.name.clone();
        let is_open = self
            .opened_menu
            .as_ref()
            .is_some_and(|name| name == &menu_name);

        let title_bar_entity = cx.entity();

        div()
            .id(menu.name.clone())
            .on_click(cx.listener(move |title_bar, _, _window, cx| {
                title_bar.opened_menu = Some(menu_name.clone());
                cx.notify();
            }))
            .hover(hover_element(theme))
            .child(menu.name.clone())
            .when(is_open, |this| {
                this.child(
                    deferred(
                        anchored()
                            .anchor(gpui::Anchor::TopLeft)
                            .snap_to_window_with_margin(px(8.))
                            .child(
                                popover()
                                    .on_mouse_down_out(cx.listener(|title_bar, _, _, cx| {
                                        title_bar.opened_menu = None;
                                        cx.notify();
                                    }))
                                    .bg(theme.surface())
                                    .items_start()
                                    .children(menu.items.iter().map(|menu_item| match menu_item {
                                        gpui::OwnedMenuItem::Separator => {
                                            div().child("-").into_any()
                                        }
                                        gpui::OwnedMenuItem::Submenu(owned_menu) => {
                                            div().child(owned_menu.name.clone()).into_any()
                                        }
                                        gpui::OwnedMenuItem::SystemMenu(owned_os_menu) => {
                                            div().child(owned_os_menu.name.clone()).into_any()
                                        }
                                        gpui::OwnedMenuItem::Action {
                                            name,
                                            action,
                                            os_action: _,
                                            checked: _,
                                            disabled,
                                        } => {
                                            let action = action.boxed_clone();
                                            let title_bar_entity = title_bar_entity.clone();
                                            div()
                                                .id(name.clone())
                                                .child(name.to_string())
                                                .cursor_pointer()
                                                .hover(hover_element(theme))
                                                .when(!disabled, |this| {
                                                    this.on_click(move |_event, window, app| {
                                                        window.dispatch_action(
                                                            action.boxed_clone(),
                                                            app,
                                                        );
                                                        title_bar_entity.update(
                                                            app,
                                                            |title_bar, cx| {
                                                                title_bar.opened_menu = None;
                                                                cx.notify()
                                                            },
                                                        )
                                                    })
                                                })
                                                .into_any()
                                        }
                                    })),
                            ),
                    )
                    .priority(2),
                )
            })
    }
}

fn popover() -> gpui::Stateful<Div> {
    div()
        .id("popover")
        .occlude()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .shadow_lg()
        .p_3()
        .rounded_md()
        .bg(gpui::white())
        .text_color(gpui::black())
        .border_1()
        .text_sm()
        .border_color(gpui::black().opacity(0.1))
}

impl Render for TitleBar {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme: Theme = window.appearance().into();

        let button_hover = |style: StyleRefinement| hover_element(theme)(style.cursor_pointer());

        // macOS draws native traffic lights over the (transparent) titlebar on the
        // left, so reserve space for them and skip our own window controls.
        let is_macos = cfg!(target_os = "macos");
        // In fullscreen the traffic lights are hidden, so the inset is no longer needed.
        let reserve_traffic_lights = is_macos && !window.is_fullscreen();

        div()
            .bg(theme.title_bar_bg())
            .h(px(32.))
            .p_2()
            // Inset the title so it clears the native traffic lights on macOS.
            .when(reserve_traffic_lights, |this| this.pl(px(72.)))
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .when(!is_macos, |this| this.child(self.render_menus(cx, theme)))
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
                    }), // .child("Hacker News Dashboard")
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
