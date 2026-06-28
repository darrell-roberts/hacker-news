//! Simple hacker news view.
use crate::{common::save_config, main_window::WindowResize};
use gpui::{
    App, Bounds, Global, Menu, MenuItem, SharedString, WindowBounds, WindowDecorations, WindowKind,
    WindowOptions, actions, point, px, size,
};
use gpui_platform::application;
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
        app.set_global(ApiClientState(client));
        app.set_global(ArticleSelection {
            viewing_article_type: ArticleType::Top,
            viewing_article_total: 50,
        });
        app.set_global(UrlHover(None));
        app.set_global(config);

        app.activate(true);
        app.on_action(quit);

        // Add menu items
        app.set_menus([Menu::new("set_menus").items([MenuItem::action("Quit", Quit)])]);

        app.on_window_closed(|app, _window_id| {
            app.quit();
        })
        .detach();

        // Write back changes made to config to disk.
        app.observe_global::<Config>(|cx| {
            let config = *cx.global::<Config>();
            cx.spawn(async move |_app| {
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
actions!(set_menus, [Quit]);

// Define the quit function that is registered with the AppContext
fn quit(_: &Quit, cx: &mut App) {
    info!("Gracefully quitting the application...");
    cx.quit();
}
