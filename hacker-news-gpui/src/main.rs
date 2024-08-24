//! Simple hacker news view.
use content::Content;
use gpui::{
    actions, div, prelude::*, px, rgb, size, App, AppContext, Global, Menu, MenuItem, SharedString,
    View, WindowContext, WindowOptions,
};
use hacker_news_api::ApiClient;
use header::Header;
use std::sync::Arc;

mod article;
mod content;
mod header;

#[derive(Clone)]
pub struct ApiClientState(Arc<ApiClient>);

impl Global for ApiClientState {}

struct MainWindow {
    header: View<Header>,
    content: View<Content>,
}

impl MainWindow {
    fn new(cx: &mut WindowContext) -> View<Self> {
        let header = Header::new(cx);
        let content = Content::new(cx);

        cx.new_view(|_| Self { header, content })
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
            .child(self.header.clone())
            .child(self.content.clone())
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

        let bounds = gpui::Bounds::centered(None, size(px(800.), px(600.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Hacker News".into()),
                    ..Default::default()
                }),
                window_decorations: Some(Default::default()),
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
