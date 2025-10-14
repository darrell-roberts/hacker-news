//! Application configuration.
use hacker_news_api::ArticleType;
use hacker_news_search::IndexStats;
use serde::{Deserialize, Serialize};

pub const CONFIG_FILE: &str = "config_tui.dat";

/// Application configuration.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Config {
    /// Stats for each index.
    pub index_stats: Vec<IndexStats>,
    /// Active index.
    pub active_index: ArticleType,
}
