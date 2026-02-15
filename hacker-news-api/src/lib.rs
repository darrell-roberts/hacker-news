//! A simple API client for the Hacker News firebase API.
mod client;
mod types;

pub use crate::client::{subscribe_to_article_list, ApiClient};
use thiserror::Error;
pub use types::{ArticleType, Item, ItemEventData, ResultExt, StoriesEventData, User};

#[derive(Debug, Error, Default)]
pub enum ApiError {
    #[default]
    #[error("Stream collection failed")]
    Default,
    #[error("Http error {0}")]
    Http(#[from] reqwest::Error),
}
