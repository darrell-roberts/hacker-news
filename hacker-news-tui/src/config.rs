//! Application configuration.
use hacker_news_config::{INDEX_CONFIG, IndexConfig};
use serde::{Deserialize, Serialize};

/// Application configuration.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Config {
    pub index_config: IndexConfig,
}

/// Save application configuration.
pub fn save_config(config: Config) -> impl Future<Output = anyhow::Result<()>> {
    let Config { index_config } = config;
    hacker_news_config::save_config(index_config, INDEX_CONFIG)
}

/// Load application configuration.
pub fn load_config() -> anyhow::Result<Config> {
    let index_config = hacker_news_config::load_config(INDEX_CONFIG)?;
    Ok(Config { index_config })
}
