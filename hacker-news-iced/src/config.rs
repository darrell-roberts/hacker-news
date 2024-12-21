use crate::{
    app::{App, PaneState},
    articles::ArticleState,
    footer::FooterState,
    full_search::FullSearchState,
    header::HeaderState,
    theme,
};
use anyhow::Context;
use app_dirs2::{get_app_root, AppDataType, AppInfo};
use hacker_news_api::ArticleType;
use hacker_news_search::{IndexStats, SearchContext};
use iced::{
    widget::pane_grid::{self, Configuration},
    Size,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

pub const APP_INFO: AppInfo = AppInfo {
    name: "Hacker News",
    author: "Somebody",
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub scale: f64,
    pub article_count: usize,
    pub article_type: ArticleType,
    pub visited: HashSet<u64>,
    pub theme: String,
    pub window_size: (f32, f32),
    pub current_index_stats: Option<IndexStats>,
    pub index_stats: Vec<IndexStats>,
}

pub async fn save_config(config: Config) -> anyhow::Result<()> {
    let config_dir = get_app_root(AppDataType::UserConfig, &APP_INFO).context("No app root")?;
    if !config_dir.exists() {
        tokio::fs::create_dir_all(&config_dir).await?;
    }

    let contents = rmp_serde::to_vec(&config)?;
    let config_path = config_dir.join("config.dat");

    tokio::fs::write(&config_path, &contents).await?;

    Ok(())
}

pub fn load_config() -> anyhow::Result<Config> {
    let config_dir = get_app_root(AppDataType::UserConfig, &APP_INFO).context("No app root")?;
    let content = std::fs::read(config_dir.join("config.dat"))?;
    let config = rmp_serde::from_slice(&content)?;

    Ok(config)
}

impl Config {
    pub fn into_app(self, search_context: Arc<RwLock<SearchContext>>) -> App {
        let config = self;
        let index_stats = HashMap::from_iter(
            config
                .index_stats
                .into_iter()
                .map(|index_stat| (index_stat.category.as_str(), index_stat)),
        );
        App {
            search_context: search_context.clone(),
            theme: theme(&config.theme).unwrap_or_default(),
            scale: config.scale,
            header: HeaderState {
                search_context: search_context.clone(),
                article_count: config.article_count,
                article_type: config.article_type,
                building_index: false,
                full_search: None,
            },
            footer: FooterState {
                status_line: String::new(),
                last_update: None,
                scale: config.scale,
                current_index_stats: config.current_index_stats,
                index_stats,
                index_progress: None,
            },
            article_state: ArticleState {
                search_context: search_context.clone(),
                articles: Vec::new(),
                visited: config.visited,
                search: None,
                viewing_item: None,
                article_limit: config.article_count,
                watch_handles: HashMap::new(),
                watch_changes: HashMap::new(),
                indexing_stories: Vec::new(),
            },
            comment_state: None,
            size: Size::new(config.window_size.0, config.window_size.1),
            panes: pane_grid::State::with_configuration(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Vertical,
                ratio: 0.3,
                a: Box::new(Configuration::Pane(PaneState::Articles)),
                b: Box::new(Configuration::Pane(PaneState::Comments)),
            }),
            full_search_state: FullSearchState {
                search_context: search_context.clone(),
                search: None,
                search_results: Vec::new(),
                offset: 0,
                page: 1,
                full_count: 0,
            },
            focused_pane: None,
        }
    }
}
