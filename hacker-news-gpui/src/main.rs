//! Simple hacker news view.
use crate::{common::save_config, main_window::WindowResize};
use gpui::{
    App, BorrowAppContext, Bounds, Global, KeyBinding, Menu, MenuItem, SharedString, WindowBounds,
    WindowDecorations, WindowKind, WindowOptions, actions, point, px, size,
};
use gpui_platform::application;
use gpui_tokio::Tokio;
use hacker_news_api::{ApiClient, ArticleType};
use hacker_news_config::{init_logger, load_config};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, sync::Arc};

mod article;
mod article_body;
mod comment;
mod common;
mod content;
mod footer;
mod header;
mod main_window;
mod rich_text;
mod scrollbar;
mod theme;
mod title_bar;

const CONFIG_FILE: &str = "hacker-news-dashboard.config";

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

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    font_size: f32,
}

impl Global for Config {}

fn main() -> anyhow::Result<()> {
    init_logger("hacker-news-dashboard")?;

    let config = match load_config::<Config>(CONFIG_FILE) {
        Ok(config) => config,
        Err(_) => {
            info!("No config");
            Config { font_size: 15.0 }
        }
    };

    let client = Arc::new(hacker_news_api::ApiClient::new()?);

    application().run(move |app| {
        gpui_tokio::init(app);

        app.set_global(ApiClientState(client));
        app.set_global(ArticleSelection {
            viewing_article_type: ArticleType::Top,
            viewing_article_total: 50,
        });
        app.set_global(UrlHover(None));
        app.set_global(config);

        app.activate(true);

        // Add menu action handlers.
        app.on_action(quit);
        app.on_action(|TopTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::Top;
            });
        });
        app.on_action(|BestTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::Best;
            });
        });
        app.on_action(|NewTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::New;
            });
        });
        app.on_action(|AskTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::Ask;
            });
        });
        app.on_action(|ShowTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::Show;
            });
        });
        app.on_action(|JobTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::Job;
            });
        });

        // Bind hot keys to the actions. The menu items automatically display
        // the matching keystroke for any action that has a binding.
        #[cfg(target_os = "macos")]
        app.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-1", TopTopic, None),
            KeyBinding::new("cmd-2", BestTopic, None),
            KeyBinding::new("cmd-3", NewTopic, None),
            KeyBinding::new("cmd-4", AskTopic, None),
            KeyBinding::new("cmd-5", ShowTopic, None),
            KeyBinding::new("cmd-6", JobTopic, None),
        ]);

        #[cfg(not(target_os = "macos"))]
        app.bind_keys([
            KeyBinding::new("ctrl-q", Quit, None),
            KeyBinding::new("ctrl-1", TopTopic, None),
            KeyBinding::new("ctrl-2", BestTopic, None),
            KeyBinding::new("ctrl-3", NewTopic, None),
            KeyBinding::new("ctrl-4", AskTopic, None),
            KeyBinding::new("ctrl-5", ShowTopic, None),
            KeyBinding::new("ctrl-6", JobTopic, None),
        ]);

        // Add menu items
        app.set_menus([
            Menu::new("☰").items([MenuItem::action("⏻  Quit", Quit)]),
            Menu::new("Topics").items([
                MenuItem::action("🔝  Top", TopTopic),
                MenuItem::action("⭐  Best", BestTopic),
                MenuItem::action("🆕  New", NewTopic),
                MenuItem::separator(),
                MenuItem::action("❓  Ask", AskTopic),
                MenuItem::action("📺  Show", ShowTopic),
                MenuItem::action("💼  Job", JobTopic),
            ]),
        ]);

        app.on_window_closed(|app, _window_id| {
            app.quit();
        })
        .detach();

        // Write back changes made to config to disk.
        app.observe_global::<Config>(|cx| {
            let config = *cx.global::<Config>();
            Tokio::spawn(cx, async move {
                if let Err(err) = save_config(config).await {
                    error!("Failed to save config: {err}");
                }
            })
            .detach();
        })
        .detach();

        // Clamp the preferred window size to the primary display so the window
        // never opens larger than (and therefore partially outside of) the
        // visible desktop.
        let preferred = size(px(1900.), px(1200.));
        let window_size = app.primary_display().map_or(preferred, |display| {
            let available = display.bounds().size;
            size(
                preferred.width.min(available.width),
                preferred.height.min(available.height),
            )
        });

        app.open_window(
            WindowOptions {
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Hacker News Live".into()),
                    traffic_light_position: Some(point(px(9.), px(9.))),
                    appears_transparent: true,
                }),
                window_decorations: Some(WindowDecorations::Client),
                window_min_size: Some(size(px(400.), px(800.))),
                is_movable: true,
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    window_size,
                    app,
                ))),
                show: true,
                focus: true,
                kind: WindowKind::Normal,
                app_id: Some("io.github.darrellroberts.hacker-news-dashboard".into()),
                ..Default::default()
            },
            WindowResize::new,
        )
        .expect("Could not open window");
    });

    Ok(())
}

// Associate actions using the `actions!` macro (or `impl_actions!` macro)
actions!(
    set_menus,
    [
        Quit, TopTopic, BestTopic, NewTopic, AskTopic, ShowTopic, JobTopic
    ]
);

// Define the quit function that is registered with the AppContext
fn quit(_: &Quit, cx: &mut App) {
    info!("Gracefully quitting the application...");
    cx.quit();
}
