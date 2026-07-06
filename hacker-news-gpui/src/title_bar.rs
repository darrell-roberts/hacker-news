//! Title bar.
use crate::{common::hover_element, theme::Theme};
use gpui::{
    AnyElement, App, Div, Entity, Hsla, MouseButton, OwnedMenu, Render, SharedString,
    StyleRefinement, Window, anchored, deferred, div, prelude::*, px, rems,
};

pub struct TitleBar {
    opened_menu: Option<SharedString>,
}

impl TitleBar {
    /// Create a new titlebar with title, minimize, maximize and close controls.
    pub fn new(_window: &mut Window, app: &mut App) -> Entity<Self> {
        app.new(|_cx| Self { opened_menu: None })
    }

    fn render_menus(
        &self,
        window: &Window,
        cx: &gpui::Context<Self>,
        theme: Theme,
    ) -> impl gpui::IntoElement {
        match cx.get_menus() {
            Some(menus) => div().flex().flex_row().gap_2().children(
                menus
                    .iter()
                    .map(|menu| self.menu_button(window, cx, theme, menu)),
            ),
            None => div(),
        }
    }

    fn menu_button(
        &self,
        window: &Window,
        cx: &gpui::Context<Self>,
        theme: Theme,
        menu: &OwnedMenu,
    ) -> impl IntoElement {
        let menu_name = menu.name.clone();
        let is_open = self
            .opened_menu
            .as_ref()
            .is_some_and(|name| name == &menu_name);

        div()
            .id(menu.name.clone())
            .cursor_pointer()
            .p_1()
            .on_click(cx.listener(move |title_bar, _, _window, cx| {
                title_bar.opened_menu = Some(menu_name.clone());
                cx.notify();
            }))
            .hover(hover_element(theme))
            .child(menu.name.clone())
            .when(is_open, |this| {
                let title_bar_entity = cx.entity();

                this.child(
                    deferred(
                        anchored()
                            .anchor(gpui::Anchor::TopLeft)
                            .snap_to_window_with_margin(px(8.))
                            .child(
                                popover(theme)
                                    .on_mouse_down_out(cx.listener(|title_bar, _, _, cx| {
                                        title_bar.opened_menu = None;
                                        cx.notify();
                                    }))
                                    .bg(theme.surface())
                                    .items_start()
                                    .children(render_menu_items(
                                        window,
                                        theme,
                                        menu,
                                        title_bar_entity,
                                    )),
                            ),
                    )
                    .priority(2),
                )
            })
    }
}

fn render_menu_items<'a>(
    window: &'a Window,
    theme: Theme,
    menu: &'a OwnedMenu,
    title_bar_entity: Entity<TitleBar>,
) -> impl Iterator<Item = AnyElement> + 'a {
    menu.items.iter().map(move |menu_item| {
        match menu_item {
            gpui::OwnedMenuItem::Separator => div()
                .w_full()
                .h(px(1.0))
                .mt_1()
                .mb_1()
                .bg(theme.border())
                .into_any(),

            gpui::OwnedMenuItem::Submenu(owned_menu) => {
                div().p_1().child(owned_menu.name.clone()).into_any()
            }

            gpui::OwnedMenuItem::SystemMenu(owned_os_menu) => {
                div().p_1().child(owned_os_menu.name.clone()).into_any()
            }

            gpui::OwnedMenuItem::Action {
                name,
                action,
                os_action: _,
                checked: _,
                disabled,
            } => {
                // Look up the keystroke bound to this
                // action so we can display it like the
                // native macOS menu does.
                let keystroke = window
                    .bindings_for_action(action.as_ref())
                    .last()
                    .map(|binding| {
                        binding
                            .keystrokes()
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(" ")
                    });
                let action = action.boxed_clone();
                let title_bar_entity = title_bar_entity.clone();

                div()
                    .id(name.clone())
                    .flex()
                    .flex_row()
                    .p_1()
                    .w_full()
                    .justify_between()
                    .gap_4()
                    .child(name.to_string())
                    .when_some(keystroke, |this, keystroke| {
                        this.child(
                            div()
                                .text_color(gpui::Rgba {
                                    a: 0.6,
                                    ..theme.text_color()
                                })
                                .child(keystroke),
                        )
                    })
                    .cursor_pointer()
                    .hover(hover_element(theme))
                    .when(!disabled, |this| {
                        this.on_click(move |_event, window, app| {
                            window.dispatch_action(action.boxed_clone(), app);
                            title_bar_entity.update(app, |title_bar, cx| {
                                title_bar.opened_menu = None;
                                cx.notify()
                            })
                        })
                    })
                    .into_any()
            }
        }
    })
}

fn popover(theme: Theme) -> gpui::Stateful<Div> {
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
        .bg(theme.popover_bg())
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

        let button_hover =
            |style: StyleRefinement| style.bg(brighten(theme.title_bar_bg().into(), 0.2));

        // macOS draws native traffic lights over the (transparent) titlebar on the
        // left, so reserve space for them and skip our own window controls.
        let is_macos = cfg!(target_os = "macos");
        // In fullscreen the traffic lights are hidden, so the inset is no longer needed.
        let reserve_traffic_lights = is_macos && !window.is_fullscreen();

        div()
            .bg(theme.title_bar_bg())
            .when(cfg!(target_os = "macos"), |this| this.h(px(32.)))
            .p_1()
            // Inset the title so it clears the native traffic lights on macOS.
            .when(reserve_traffic_lights, |this| this.pl(px(72.)))
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .text_size(rems(1.0))
            .when(!is_macos, |this| {
                this.child(self.render_menus(window, cx, theme))
            })
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
                    }),
            )
            .when(!is_macos, |this| {
                this.child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            /* minimize button */
                            window_control(button_hover).child("−").id("min").on_click(
                                |_, window, _cx| {
                                    window.minimize_window();
                                },
                            ),
                        )
                        .child(
                            /* maximize button */
                            window_control(button_hover)
                                .child(if window.is_maximized() { "❐" } else { "□" })
                                .id("max")
                                .on_click(|_, window, _| {
                                    window.zoom_window();
                                }),
                        )
                        .child(
                            /* close button */
                            window_control(button_hover)
                                .child("✕")
                                .id("close")
                                .on_click(|_, _, cx| cx.quit()),
                        ),
                )
            })
    }
}

fn window_control(hover: impl Fn(StyleRefinement) -> StyleRefinement) -> Div {
    div()
        .rounded_full()
        .border_2()
        .size(rems(2.0))
        .cursor_pointer()
        .text_align(gpui::TextAlign::Center)
        .hover(hover)
}

fn brighten(color: Hsla, amount: f32) -> Hsla {
    Hsla {
        l: (color.l + amount).clamp(0.0, 1.0),
        ..color
    }
}
