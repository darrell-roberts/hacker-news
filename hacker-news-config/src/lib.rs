//! Configuration for client apps.
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use anyhow::Context as _;
use app_dirs2::{get_app_dir, get_app_root, AppDataType, AppInfo};
use flexi_logger::{Age, Cleanup, Criterion, FileSpec, Naming};
use hacker_news_api::ArticleType;
use hacker_news_search::{IndexStats, SearchContext};
use log::info;
use serde::{Deserialize, Serialize};

#[cfg(target_family = "unix")]
pub mod limits;

/// Saved viewing state of the index
pub const INDEX_CONFIG: &str = "index_config.data";

/// Index configuration.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct IndexConfig {
    pub index_stats: Vec<IndexStats>,
    pub viewing_count: usize,
    pub viewing_type: ArticleType,
}

/// Application information.
pub const APP_INFO: AppInfo = AppInfo {
    name: "Hacker News",
    author: "Somebody",
};

/// Save the application configuration.
pub async fn save_config(config: impl Serialize, file_name: &str) -> anyhow::Result<()> {
    let config_dir = get_app_root(AppDataType::UserConfig, &APP_INFO).context("No app root")?;
    if !config_dir.exists() {
        tokio::fs::create_dir_all(&config_dir).await?;
    }

    let contents = rmp_serde::to_vec(&config)?;
    let config_path = config_dir.join(file_name);

    tokio::fs::write(&config_path, &contents).await?;
    info!("Wrote to config file: {config_path:?}");

    Ok(())
}

/// Load the application configuration.
pub fn load_config<C>(file_name: &str) -> anyhow::Result<C>
where
    C: for<'a> Deserialize<'a>,
{
    let config_dir = get_app_root(AppDataType::UserConfig, &APP_INFO).context("No app root")?;
    let content = std::fs::read(config_dir.join(file_name))?;
    let config = rmp_serde::from_slice(&content)?;

    Ok(config)
}

/// Get the shared log directory.
pub fn log_dir() -> anyhow::Result<PathBuf> {
    get_app_dir(app_dirs2::AppDataType::UserData, &APP_INFO, "logs")
        .context("Failed to get app logs directory")
}

pub fn init_logger() -> anyhow::Result<()> {
    let _logger = flexi_logger::Logger::try_with_env_or_str("info")?
        .log_to_file(
            FileSpec::default()
                .directory(log_dir()?)
                .basename("hacker-news"),
        )
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(5),
        )
        .print_message()
        .start()?;
    Ok(())
}

pub fn search_context() -> anyhow::Result<Arc<RwLock<SearchContext>>> {
    let index_dir = get_app_dir(
        app_dirs2::AppDataType::UserData,
        &APP_INFO,
        "hacker-news-index",
    )?;

    // info!("Reading index dir {index_dir:?}");

    let search_context = Arc::new(RwLock::new(SearchContext::new(
        &index_dir,
        ArticleType::Top,
    )?));

    Ok(search_context)
}
