//! A simple API client for the Hacker News firebase API.
mod client;
mod types;

pub use crate::client::{subscribe_top_stories, ApiClient};
use thiserror::Error;
pub use types::{ArticleType, Item, ResultExt, User};

#[derive(Debug, Error, Default)]
pub enum ApiError {
    #[default]
    #[error("Stream collection failed")]
    Default,
    #[error("Http error {0}")]
    Http(#[from] reqwest::Error),
}
