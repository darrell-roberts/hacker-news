use gpui::{
    actions, div, prelude::*, px, rgb, size, App, AppContext, Menu, MenuItem, Model, SharedString,
    View, WindowContext, WindowOptions,
};
use hacker_news_api::{ArticleType, Item};

mod article;

struct MainWindow {
    header: View<Header>,
    content: View<Content>,
}
struct Header;
struct Content {
    articles: Model<Vec<Item>>,
}

impl MainWindow {
    fn new(cx: &mut WindowContext) -> View<Self> {
        let header = Header::new(cx);
        let content = Content::new(cx);

        cx.new_view(|_| Self { header, content })
    }
}

impl Header {
    fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_| Self)
    }
}

impl Content {
    fn new(cx: &mut WindowContext) -> View<Self> {
        let articles = cx.new_model(|_| Vec::new());

        let articles_clone = articles.clone();
        cx.spawn(|mut cx| async move {
            let client = hacker_news_api::ApiClient::new().unwrap();
            let new_articles = client.articles(10, ArticleType::Top).await.unwrap();
            articles_clone.update(&mut cx, |a, m_ctx| {
                a.extend(new_articles);
            })
        })
        .detach();
        cx.new_view(|_| Self { articles })
    }
}

impl Render for MainWindow {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x2e7d32))
            .flex_col()
            .child(self.header.clone())
            .child(self.content.clone())
    }
}

impl Render for Header {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        div().flex().bg(rgb(0xffffff)).child("Header")
    }
}

impl Render for Content {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let articles = self.articles.read(cx).into_iter().map(|article| {
            let title = article.title.clone().unwrap_or_default();
            div().child(title)
        });
        div().flex().bg(rgb(0xffffff)).children(articles)
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

        let bounds = gpui::Bounds::centered(None, size(px(300.), px(300.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
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
