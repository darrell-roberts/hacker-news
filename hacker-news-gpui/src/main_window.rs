//! Main Window.
use crate::{
    Config, content::ContentView, footer::FooterView, header::Header, theme::Theme,
    title_bar::TitleBar,
};
use gpui::{
    App, AppContext, Div, Entity, MouseButton, Pixels, ResizeEdge, SharedString, Stateful, Window,
    div, prelude::*, px,
};

pub struct WindowResize {
    inner: Entity<MainWindow>,
}

impl WindowResize {
    /// Create a new resizable window.
    pub fn new(window: &mut Window, app: &mut App) -> Entity<Self> {
        let main_window = MainWindow::new(window, app);
        app.new(|_cx| Self { inner: main_window })
    }
}

impl Render for WindowResize {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .size_full()
            .child(self.inner.clone())
            // edges — thin strips along each side
            .child(
                resize_handle(ResizeEdge::Top)
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(4.)),
            )
            .child(
                resize_handle(ResizeEdge::Bottom)
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(px(4.)),
            )
            .child(
                resize_handle(ResizeEdge::Left)
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(px(4.)),
            )
            .child(
                resize_handle(ResizeEdge::Right)
                    .right_0()
                    .top_0()
                    .bottom_0()
                    .w(px(4.)),
            )
            // corners — small squares so diagonal resize is reachable
            .child(
                resize_handle(ResizeEdge::TopLeft)
                    .top_0()
                    .left_0()
                    .size(px(8.)),
            )
            .child(
                resize_handle(ResizeEdge::TopRight)
                    .top_0()
                    .right_0()
                    .size(px(8.)),
            )
            .child(
                resize_handle(ResizeEdge::BottomLeft)
                    .bottom_0()
                    .left_0()
                    .size(px(8.)),
            )
            .child(
                resize_handle(ResizeEdge::BottomRight)
                    .bottom_0()
                    .right_0()
                    .size(px(8.)),
            )
    }
}

fn resize_handle(edge: ResizeEdge) -> Stateful<Div> {
    div()
        .id(SharedString::from(format!("resize-{edge:?}")))
        .absolute()
        .hover(|style| match edge {
            ResizeEdge::Top => style.cursor_n_resize(),
            ResizeEdge::TopRight => style.cursor_nesw_resize(),
            ResizeEdge::Right => style.cursor_e_resize(),
            ResizeEdge::BottomRight => style.cursor_nwse_resize(),
            ResizeEdge::Bottom => style.cursor_s_resize(),
            ResizeEdge::BottomLeft => style.cursor_nesw_resize(),
            ResizeEdge::Left => style.cursor_w_resize(),
            ResizeEdge::TopLeft => style.cursor_nwse_resize(),
        })
        .on_mouse_down(MouseButton::Left, move |_, window, _| {
            window.start_window_resize(edge);
        })
}

/// The main window view.
pub struct MainWindow {
    header: Entity<Header>,
    content: Entity<ContentView>,
    footer: Entity<FooterView>,
    base_font_size: Pixels,
    title_bar: Entity<TitleBar>,
}

impl MainWindow {
    /// Create the main window.
    ///
    /// # Arguments
    ///
    /// * `window` - A mutable reference to the Window in which the main UI will be created.
    /// * `app` - A mutable reference to the App, used for managing application state and actions.
    ///
    pub fn new(window: &mut Window, app: &mut App) -> Entity<Self> {
        let header = Header::new(window, app);
        let content = ContentView::new(window, app);
        let footer = FooterView::new(window, app, content.clone());
        let title_bar = TitleBar::new(window, app);

        let font_size = app.global::<Config>().font_size;

        // Listen to system theme changes.
        window
            .observe_window_appearance(|_window, app| {
                app.refresh_windows();
            })
            .detach();

        app.new(move |cx| {
            // Listen to font size increase/decrease key bindings.
            cx.observe_keystrokes(|main_window: &mut MainWindow, event, window, cx| {
                let mut adjust_text_size = |val| {
                    main_window.base_font_size =
                        (main_window.base_font_size + px(val)).clamp(px(10.), px(35.));
                    window.set_rem_size(main_window.base_font_size);
                    let font_size = main_window.base_font_size.as_f32();

                    cx.update_global::<Config, _>(|config, _cx| {
                        config.font_size = font_size;
                    });

                    cx.notify();
                };

                if event.keystroke.modifiers.control {
                    match event.keystroke.key.as_str() {
                        "add" | "+" => {
                            adjust_text_size(1.);
                        }
                        "subtract" | "-" => {
                            adjust_text_size(-1.);
                        }
                        _ => {}
                    }
                }
            })
            .detach();

            Self {
                title_bar,
                header,
                content,
                footer,
                base_font_size: px(font_size),
            }
        })
    }
}

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme: Theme = window.appearance().into();

        div()
            .id("main_window")
            .font_family(".SystemUIFont")
            .text_size(self.base_font_size)
            .text_color(theme.text_color())
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme.bg())
            .child(self.title_bar.clone())
            // .child(self.header.clone())
            .child(div().flex_1().min_h_0().child(self.content.clone()))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .mt_auto()
                    .child(self.footer.clone()),
            )
    }
}
