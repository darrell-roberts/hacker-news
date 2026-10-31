//! Main Window.
use crate::{
    Config, DefaultFonts, content::ContentView, footer::FooterView, header::Header, theme::Theme,
    title_bar::TitleBar,
};
use gpui::{
    App, AppContext, Div, Entity, MouseButton, Pixels, ResizeEdge, SharedString, Stateful, Window,
    div, prelude::*, px,
};
use platform_settings::FontData;

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
    #[expect(dead_code)]
    header: Entity<Header>,
    content: Entity<ContentView>,
    footer: Entity<FooterView>,
    base_font_size: Pixels,
    // base_font_name: SharedString,
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
        let title_bar = TitleBar::new(window, app, "Hacker News Dashboard".into());

        let font_size = app.global::<DefaultFonts>().system_ui.size;

        // Listen to system theme changes.
        window
            .observe_window_appearance(|_window, app| {
                app.refresh_windows();
            })
            .detach();

        #[cfg(target_os = "linux")]
        subscribe_to_system_font_changes(app);

        app.new(move |cx| {
            cx.observe_global::<DefaultFonts>({
                move |main_window: &mut MainWindow, cx| {
                    let fonts = cx.global::<DefaultFonts>();
                    main_window.base_font_size = fonts.system_ui.size;
                }
            })
            .detach();

            // Listen to font size increase/decrease key bindings.
            cx.observe_keystrokes(|main_window: &mut MainWindow, event, _window, cx| {
                let mut adjust_text_size = |val| {
                    let new_size = (main_window.base_font_size + px(val)).clamp(px(10.), px(35.));

                    cx.update_global(|fonts: &mut DefaultFonts, _cx| {
                        fonts.system_ui.size = new_size;
                        fonts.monospace.size = new_size;
                    });

                    cx.update_global(|config: &mut Config, _cx| {
                        config.font_size = new_size.as_f32();
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
                base_font_size: font_size,
            }
        })
    }
}

#[cfg(target_os = "linux")]
fn subscribe_to_system_font_changes(app: &mut App) {
    use futures::channel::mpsc::channel;
    use gpui::AsyncApp;

    #[derive(Debug)]
    enum FontType {
        SystemUi(FontData),
        Monospace(FontData),
        Scale(f64),
    }
    let (mut tx, mut rx) = channel::<FontType>(10);

    app.spawn(async move |async_app: &mut AsyncApp| {
        use futures::StreamExt as _;

        while let Some(font_data) = rx.next().await {
            log::info!("New font data {font_data:#?}");
            async_app.update_global(|default_fonts: &mut DefaultFonts, _app| {
                match font_data {
                    FontType::SystemUi(font_data) => {
                        default_fonts.system_ui = font_data.into();
                    }
                    FontType::Monospace(font_data) => {
                        default_fonts.monospace = font_data.into();
                    }
                    // GPUI handles scale for us.
                    FontType::Scale(_scale) => {}
                };
            });
        }
    })
    .detach();

    app.background_spawn(async move {
        use futures::StreamExt;

        let mut stream = platform_settings::listen_to_system_changes();
        while let Some(event) = stream.next().await {
            match event {
                platform_settings::SettingChange::Theme(_theme) => {}
                platform_settings::SettingChange::FontScale(scale) => {
                    if let Err(err) = tx.try_send(FontType::Scale(scale)) {
                        log::error!("Failed to send font data update: {err}");
                    }
                }
                platform_settings::SettingChange::SystemUIFont(font_data) => {
                    if let Err(err) = tx.try_send(FontType::SystemUi(font_data)) {
                        log::error!("Failed to send font data update: {err}");
                    }
                }
                platform_settings::SettingChange::SystemMonospaceFont(font_data) => {
                    if let Err(err) = tx.try_send(FontType::Monospace(font_data)) {
                        log::error!("Failed to send font data update: {err}");
                    }
                }
            }
        }
    })
    .detach();

    app.observe_global::<DefaultFonts>(|app| {
        app.refresh_windows();
    })
    .detach();
}

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme: Theme = window.appearance().into();
        let base_font = &cx.global::<DefaultFonts>().system_ui;
        window.set_rem_size(self.base_font_size);

        div()
            .id("main_window")
            .font_family(base_font.family_name.clone())
            .text_size(base_font.size)
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
