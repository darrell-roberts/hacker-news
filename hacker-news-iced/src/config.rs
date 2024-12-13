use anyhow::Context;
use app_dirs2::{get_app_root, AppDataType, AppInfo};
use hacker_news_api::ArticleType;
use hacker_news_search::IndexStats;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
