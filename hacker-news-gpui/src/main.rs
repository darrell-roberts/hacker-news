//! Simple hacker news view.
use crate::{common::save_config, main_window::WindowResize};
use gpui::{
    Action, App, BorrowAppContext, Bounds, Global, KeyBinding, Menu, MenuItem, SharedString,
    WindowBounds, WindowDecorations, WindowKind, WindowOptions, actions, point, px, size,
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

impl ArticleSelection {
    /// Article selection as the menu actions.
    fn as_menu_actions(&self) -> [&dyn Action; 2] {
        [
            match self.viewing_article_type {
                ArticleType::New => &NewTopic,
                ArticleType::Best => &BestTopic,
                ArticleType::Top => &TopTopic,
                ArticleType::Ask => &AskTopic,
                ArticleType::Show => &ShowTopic,
                ArticleType::Job => &JobTopic,
            },
            match self.viewing_article_total {
                25 => &ArticleLimit25,
                50 => &ArticleLimit50,
                75 => &ArticleLimit75,
                _ => unreachable!("only 25, 50, 75"),
            },
        ]
    }
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

fn build_menus(
    selected_topic: &dyn Action,
    selected_limit: &dyn Action,
) -> impl IntoIterator<Item = Menu> {
    [Menu::new("☰").items([
        MenuItem::submenu(Menu::new("Topics").items([
            MenuItem::action("🔝  Top", TopTopic).checked(TopTopic.partial_eq(selected_topic)),
            MenuItem::action("⭐  Best", BestTopic).checked(BestTopic.partial_eq(selected_topic)),
            MenuItem::action("🆕  New", NewTopic).checked(NewTopic.partial_eq(selected_topic)),
            MenuItem::separator(),
            MenuItem::action("❓  Ask", AskTopic).checked(AskTopic.partial_eq(selected_topic)),
            MenuItem::action("📺  Show", ShowTopic).checked(ShowTopic.partial_eq(selected_topic)),
            MenuItem::action("💼  Job", JobTopic).checked(JobTopic.partial_eq(selected_topic)),
        ])),
        MenuItem::Separator,
        MenuItem::Submenu(
            Menu::new("Limit").items([
                MenuItem::action("25", ArticleLimit25)
                    .checked(ArticleLimit25.partial_eq(selected_limit)),
                MenuItem::action("50", ArticleLimit50)
                    .checked(ArticleLimit50.partial_eq(selected_limit)),
                MenuItem::action("75", ArticleLimit75)
                    .checked(ArticleLimit75.partial_eq(selected_limit)),
            ]),
        ),
        MenuItem::Separator,
        MenuItem::action("⏻  Quit", Quit),
    ])]
}

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
        app.on_action(|_: &TopTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::Top;
            });
            update_menus(app);
        });
        app.on_action(|_: &BestTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::Best;
            });
            update_menus(app);
        });
        app.on_action(|_: &NewTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::New;
            });
            update_menus(app);
        });
        app.on_action(|_: &AskTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::Ask;
            });
            update_menus(app);
        });
        app.on_action(|_: &ShowTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::Show;
            });
            update_menus(app);
        });
        app.on_action(|_: &JobTopic, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_type = ArticleType::Job;
            });
            update_menus(app);
        });

        // Limits
        app.on_action(|_: &ArticleLimit25, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_total = 25;
            });
            update_menus(app);
        });

        app.on_action(|_: &ArticleLimit50, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_total = 50;
            });
            update_menus(app);
        });

        app.on_action(|_: &ArticleLimit75, app| {
            app.update_global(|state: &mut ArticleSelection, _cx| {
                state.viewing_article_total = 75;
            });
            update_menus(app);
        });

        // Bind hot keys to the actions. The menu items automatically display
        // the matching keystroke for any action that has a binding.
        app.bind_keys([
            KeyBinding::new("secondary-q", Quit, None),
            KeyBinding::new("secondary-1", TopTopic, None),
            KeyBinding::new("secondary-2", BestTopic, None),
            KeyBinding::new("secondary-3", NewTopic, None),
            KeyBinding::new("secondary-4", AskTopic, None),
            KeyBinding::new("secondary-5", ShowTopic, None),
            KeyBinding::new("secondary-6", JobTopic, None),
        ]);

        // Add menu items
        app.set_menus(build_menus(&TopTopic, &ArticleLimit50));

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
        Quit,
        TopTopic,
        BestTopic,
        NewTopic,
        AskTopic,
        ShowTopic,
        JobTopic,
        ArticleLimit25,
        ArticleLimit50,
        ArticleLimit75,
    ]
);

// Define the quit function that is registered with the AppContext
fn quit(_: &Quit, cx: &mut App) {
    info!("Gracefully quitting the application...");
    cx.quit();
}

// After updating the article selection state rebuild the main menu and
// set checked selection for actions.
fn update_menus(app: &mut App) {
    let state = app.global::<ArticleSelection>();
    let [selected_topic, selected_limit] = state.as_menu_actions();
    app.set_menus(build_menus(selected_topic, selected_limit));
}
