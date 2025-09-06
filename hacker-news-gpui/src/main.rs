//! Simple hacker news view.
use content::Content;
use footer::Footer;
use gpui::{
    actions, black, div, point, prelude::*, px, size, App, Application, Entity, Global, Menu,
    MenuItem, SharedString, Window, WindowDecorations, WindowOptions,
};
use hacker_news_api::{ApiClient, ArticleType};
use header::Header;
use std::{ops::Deref, sync::Arc};

mod article;
mod comment;
mod content;
mod footer;
mod header;

#[derive(Clone)]
pub struct ApiClientState(Arc<ApiClient>);

impl Deref for ApiClientState {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Global for ApiClientState {}

pub struct AppState {
    pub viewing_article_type: ArticleType,
    pub viewing_article_total: usize,
    pub status_line: String,
}

impl Global for AppState {}

struct MainWindow {
    header: Entity<Header>,
    content: Entity<Content>,
    footer: Entity<Footer>,
    // _quit_subscription: Subscription,
}

impl MainWindow {
    fn new(window: &mut Window, app: &mut App) -> Entity<Self> {
        let header = Header::new(window, app);
        let content = Content::new(window, app);
        let footer = Footer::new(window, app, &content);

        // let subscription = cx.on_action(, listener)

        app.new(|_ctx| Self {
            header,
            content,
            footer,
        })
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(black())
            .border_5()
            .child(self.header.clone())
            .child(self.content.clone())
            .child(self.footer.clone())
    }
}

fn main() {
    Application::new().run(|app| {
        let client = Arc::new(hacker_news_api::ApiClient::new().unwrap());
        app.set_global(ApiClientState(client));
        app.set_global(AppState {
            viewing_article_type: ArticleType::Top,
            viewing_article_total: 50,
            status_line: String::from("Loading..."),
        });

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
                    title: Some("Hacker News".into()),
                    traffic_light_position: Some(point(px(9.), px(9.))),
                    ..Default::default()
                }),
                window_decorations: Some(WindowDecorations::Server),
                window_min_size: Some(size(px(800.), px(600.))),
                is_movable: true,
                window_bounds: None,
                ..Default::default()
            },
            MainWindow::new,
        )
        .unwrap();
    });
}

// Associate actions using the `actions!` macro (or `impl_actions!` macro)
actions!(set_menus, [Quit]);

// Define the quit function that is registered with the AppContext
fn _quit(_: &Quit, cx: &mut App) {
    println!("Gracefully quitting the application . . .");
    cx.quit();
}
