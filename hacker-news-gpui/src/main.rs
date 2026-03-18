//! Simple hacker news view.
use crate::{content::start_background_article_list_subscription, header::Header, theme::Theme};
use content::ContentView;
use footer::FooterView;
use gpui::{
    App, AppContext, Application, Bounds, Entity, Global, Menu, MenuItem, Pixels, SharedString,
    Window, WindowBounds, WindowDecorations, WindowKind, WindowOptions, actions, div, point,
    prelude::*, px, size,
};
use hacker_news_api::{ApiClient, ArticleType};
use hacker_news_config::init_logger;
use log::{error, info};
use std::{ops::Deref, sync::Arc};

mod article;
mod comment;
mod common;
mod content;
mod footer;
mod header;
mod rich_text;
mod scrollbar;
mod theme;

#[derive(Clone)]
/// Wrapper for ApiClient so we can put it in global gpui app state.
pub struct ApiClientState(Arc<ApiClient>);

impl Deref for ApiClientState {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Global for ApiClientState {}

#[derive(Debug, Copy, Clone)]
/// The current selection for article category and total
pub struct ArticleSelection {
    /// Article category.
    pub viewing_article_type: ArticleType,
    /// Total articles to view.
    pub viewing_article_total: usize,
}

impl Global for ArticleSelection {}

/// Global state of url hover.
pub struct UrlHover(pub Option<SharedString>);

impl Global for UrlHover {}

/// The main window view.
struct MainWindow {
    header: Entity<Header>,
    content: Entity<ContentView>,
    footer: Entity<FooterView>,
    base_font_size: Pixels,
}

impl MainWindow {
    /// Create the main window.
    ///
    /// # Arguments
    ///
    /// * `window` - A mutable reference to the Window in which the main UI will be created.
    /// * `app` - A mutable reference to the App, used for managing application state and actions.
    ///
    fn new(window: &mut Window, app: &mut App) -> Entity<Self> {
        let header = Header::new(window, app);
        let content = ContentView::new(window, app);
        let footer = FooterView::new(window, app, content.clone());

        window
            .observe_window_appearance(|_window, app| {
                app.refresh_windows();
            })
            .detach();

        let content_update = content.clone();
        app.new(move |cx| {
            cx.observe_global::<ArticleSelection>(move |main_window: &mut MainWindow, cx| {
                let selection = *cx.global::<ArticleSelection>();
                // Reset ranks when we change selection.
                main_window.content.update(cx, |content, cx| {
                    content.article_ranks.clear();
                    cx.notify();
                });
                content_update.update(cx, |content_view, cx| {
                    match content_view.article_sender.as_ref() {
                        Some(tx) => {
                            info!("Opening stream for {selection:?}");
                            let old_task = content_view.background_task.replace(
                                start_background_article_list_subscription(cx, tx.clone()),
                            );
                            if let Some(old_task) = old_task {
                                info!("dropping old task");
                                drop(old_task);
                            }
                        }
                        None => {
                            error!("No article sender on content view");
                        }
                    }
                });
            })
            .detach();

            cx.observe_keystrokes(|main_window, event, window, cx| {
                let mut adjust_text_size = |val| {
                    main_window.base_font_size =
                        (main_window.base_font_size + px(val)).clamp(px(10.), px(35.));
                    window.set_rem_size(main_window.base_font_size);
                    cx.notify();
                };

                info!("key: {event:?}");
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
                header,
                content,
                footer,
                base_font_size: px(15.),
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
            .child(self.header.clone())
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

fn main() {
    init_logger("hacker-news-dashboard").expect("Failed to setup logger");

    Application::new().run(|app| {
        let client = Arc::new(hacker_news_api::ApiClient::new().expect("No API Client"));
        app.set_global(ApiClientState(client));
        app.set_global(ArticleSelection {
            viewing_article_type: ArticleType::Top,
            viewing_article_total: 50,
        });
        app.set_global(UrlHover(None));

        // Add menu items
        app.set_menus(vec![Menu {
            name: SharedString::from("set_menus"),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        app.on_window_closed(|app| {
            app.quit();
        })
        .detach();

        app.open_window(
            WindowOptions {
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Hacker News Live".into()),
                    traffic_light_position: Some(point(px(9.), px(9.))),
                    appears_transparent: false,
                }),
                window_decorations: Some(WindowDecorations::Server),
                window_min_size: Some(size(px(400.), px(800.))),
                is_movable: true,
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1900.), px(1200.)),
                    app,
                ))),
                show: true,
                focus: true,
                kind: WindowKind::Normal,
                app_id: Some("io.github.darrellroberts.hacker-news-dashboard".into()),
                ..Default::default()
            },
            MainWindow::new,
        )
        .expect("Could not open window");
    });
}

// Associate actions using the `actions!` macro (or `impl_actions!` macro)
actions!(set_menus, [Quit]);

// Define the quit function that is registered with the AppContext
fn _quit(_: &Quit, cx: &mut App) {
    info!("Gracefully quitting the application . . .");
    cx.quit();
}
