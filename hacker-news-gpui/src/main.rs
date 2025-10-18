//! Simple hacker news view.
use crate::theme::Theme;
use content::Content;
use footer::Footer;
use gpui::{
    actions, div, point, prelude::*, px, size, App, Application, Entity, Global, Menu, MenuItem,
    SharedString, Window, WindowDecorations, WindowKind, WindowOptions,
};
use hacker_news_api::{ApiClient, ArticleType, Item};
use log::info;
use std::{ops::Deref, sync::Arc};

mod article;
mod comment;
mod common;
mod content;
mod footer;
mod theme;
// mod header;

#[derive(Clone)]
pub struct ApiClientState(Arc<ApiClient>);

impl Deref for ApiClientState {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Global for ApiClientState {}

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
    // header: Entity<Header>,
    content: Entity<Content>,
    footer: Entity<Footer>,
}

impl MainWindow {
    fn new(window: &mut Window, app: &mut App) -> Entity<Self> {
        // let header = Header::new(window, app);
        let content = Content::new(window, app);
        let footer = Footer::new(window, app, content.clone());

        app.new(|_ctx| Self {
            // header,
            content,
            footer,
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
            // .font_family(".SystemUIFont")
            .font_family("Arial")
            .text_size(px(15.))
            .text_color(theme.text_color())
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme.bg())
            .child(self.content.clone())
            .child(self.footer.clone())
    }
}

fn main() {
    flexi_logger::Logger::try_with_env()
        .unwrap()
        .start()
        .expect("Application logger");

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
                window_min_size: Some(size(px(600.), px(800.))),
                is_movable: true,
                window_bounds: None,
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
