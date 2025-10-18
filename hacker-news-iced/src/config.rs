//! Persistent configuration for saving various settings and states between launches.
use crate::{
    app::{App, PaneState},
    articles::ArticleState,
    footer::FooterState,
    header::HeaderState,
    nav_history::Content,
    theme,
};
use hacker_news_config::{IndexConfig, INDEX_CONFIG};
use hacker_news_search::SearchContext;
use iced::{
    widget::pane_grid::{self, Configuration},
    Size,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

pub const CONFIG_FILE: &str = "config_gui.dat";

#[derive(Serialize, Deserialize, Debug)]
pub struct GuiConfig {
    pub scale: f64,
    pub visited: HashSet<u64>,
    pub theme: String,
    pub window_size: (f32, f32),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub index_config: IndexConfig,
    pub gui_config: GuiConfig,
}

/// Save application configuration.
pub async fn save_config(config: Config) -> anyhow::Result<()> {
    let Config {
        index_config,
        gui_config,
    } = config;
    hacker_news_config::save_config(index_config, INDEX_CONFIG).await?;
    hacker_news_config::save_config(gui_config, CONFIG_FILE).await?;
    Ok(())
}

/// Load application configuration.
pub fn load_config() -> anyhow::Result<Config> {
    let index_config = hacker_news_config::load_config(INDEX_CONFIG)?;
    let gui_config = hacker_news_config::load_config(CONFIG_FILE)?;

    Ok(Config {
        index_config,
        gui_config,
    })
}

impl Config {
    pub fn into_app(self, search_context: Arc<RwLock<SearchContext>>) -> App {
        let config = self;
        let index_stats = HashMap::from_iter(
            config
                .index_config
                .index_stats
                .into_iter()
                .map(|index_stat| (index_stat.category, index_stat)),
        );
        App {
            search_context: search_context.clone(),
            theme: theme(&config.gui_config.theme).unwrap_or_default(),
            scale: config.gui_config.scale,
            header: HeaderState {
                search_context: search_context.clone(),
                article_count: config.index_config.viewing_count,
                article_type: config.index_config.viewing_type,
                building_index: false,
                full_search: None,
            },
            footer: FooterState {
                status_line: String::new(),
                last_update: None,
                scale: config.gui_config.scale,
                viewing_index: config.index_config.viewing_type,
                index_stats,
                index_progress: None,
            },
            article_state: ArticleState {
                search_context: search_context.clone(),
                articles: Vec::new(),
                visited: config.gui_config.visited,
                search: None,
                viewing_item: None,
                article_limit: config.index_config.viewing_count,
                watch_handles: HashMap::new(),
                watch_changes: HashMap::new(),
                indexing_stories: Vec::new(),
                filter_watching: false,
            },
            size: Size::new(
                config.gui_config.window_size.0,
                config.gui_config.window_size.1,
            ),
            panes: pane_grid::State::with_configuration(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Vertical,
                ratio: 0.3,
                a: Box::new(Configuration::Pane(PaneState::Articles)),
                b: Box::new(Configuration::Pane(PaneState::Content)),
            }),
            focused_pane: None,
            content: Content::Empty(config.index_config.viewing_type),
            history: Vec::new(),
        }
    }
}
