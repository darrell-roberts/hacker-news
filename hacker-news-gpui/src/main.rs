//! Simple hacker news view.
use content::Content;
use footer::Footer;
use gpui::{
    actions, div, point, prelude::*, px, rgb, size, App, AppContext, Global, Menu, MenuItem,
    SharedString, View, WindowContext, WindowDecorations, WindowOptions,
};
use hacker_news_api::{ApiClient, ArticleType};
use header::Header;
use std::{ops::Deref, sync::Arc};

mod article;
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
    header: View<Header>,
    content: View<Content>,
    footer: View<Footer>,
}

impl MainWindow {
    fn new(cx: &mut WindowContext) -> View<Self> {
        let header = Header::new(cx);
        let content = Content::new(cx);
        let footer = Footer::new(cx);

        cx.new_view(|_| Self {
            header,
            content,
            footer,
        })
    }
}

impl Render for MainWindow {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(0xFFFFFF))
            .border_5()
            .border_color(rgb(0xEEEEEE))
            .child(self.header.clone())
            .child(self.content.clone())
            .child(self.footer.clone())
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.activate(true);
        cx.on_action(quit);

        // Add menu items
        cx.set_menus(vec![Menu {
            name: SharedString::from("set_menus"),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        let client = Arc::new(hacker_news_api::ApiClient::new().unwrap());
        cx.set_global(ApiClientState(client));
        cx.set_global(AppState {
            viewing_article_type: ArticleType::Top,
            viewing_article_total: 50,
            status_line: String::new(),
        });

        // let bounds = gpui::Bounds::centered(None, size(px(800.), px(600.)), cx);
        cx.open_window(
            WindowOptions {
                // window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
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
            |cx| MainWindow::new(cx),
        )
        .unwrap();
    })
}

// Associate actions using the `actions!` macro (or `impl_actions!` macro)
actions!(set_menus, [Quit]);

// Define the quit function that is registered with the AppContext
fn quit(_: &Quit, cx: &mut AppContext) {
    println!("Gracefully quitting the application . . .");
    cx.quit();
}
