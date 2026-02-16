//! Simple hacker news view.
use crate::{content::start_background_article_list_subscription, header::Header, theme::Theme};
use content::ContentView;
use footer::FooterView;
use gpui::{
    actions, div, point, prelude::*, px, size, App, AppContext, Application, Bounds, Entity,
    Global, Menu, MenuItem, SharedString, Window, WindowBounds, WindowDecorations, WindowKind,
    WindowOptions,
};
use hacker_news_api::{ApiClient, ArticleType, Item};
use hacker_news_config::init_logger;
use log::info;
use std::{ops::Deref, sync::Arc};

mod article;
mod comment;
mod common;
mod content;
mod footer;
mod header;
mod theme;

#[derive(Clone)]
pub struct ApiClientState(Arc<ApiClient>);

impl Deref for ApiClientState {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Global for ApiClientState {}

#[derive(Debug, Copy, Clone)]
pub struct ArticleSelection {
    pub viewing_article_type: ArticleType,
    pub viewing_article_total: usize,
}

impl Global for ArticleSelection {}

pub struct UrlHover(pub Option<SharedString>);

impl Global for UrlHover {}

pub struct ArticleState(pub Vec<Item>);

impl Global for ArticleState {}

struct MainWindow {
    header: Entity<Header>,
    content: Entity<ContentView>,
    footer: Entity<FooterView>,
}

impl MainWindow {
    fn new(window: &mut Window, app: &mut App) -> Entity<Self> {
        let header = Header::new(window, app);
        let content = ContentView::new(window, app);
        let footer = FooterView::new(window, app, content.clone());

        let content_update = content.clone();
        app.new(move |cx| {
            cx.observe_global::<ArticleSelection>(move |_main_window: &mut MainWindow, cx| {
                let selection = *cx.global::<ArticleSelection>();
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
                            panic!("No article sender on content view");
                        }
                    }
                });
            })
            .detach();

            Self {
                header,
                content,
                footer,
            }
        })
    }
}

impl Render for MainWindow {
    fn render(&mut self, window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        window
            .observe_window_appearance(|_window, app| {
                app.refresh_windows();
            })
            .detach();

        let theme: Theme = window.appearance().into();

        div()
            .font_family(".SystemUIFont")
            .text_size(px(17.))
            .text_color(theme.text_color())
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme.bg())
            .child(self.header.clone())
            .child(self.content.clone())
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
            viewing_article_total: 25,
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
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered_at(
                    point(px(0.), px(0.)),
                    size(px(1000.), px(1200.)),
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
