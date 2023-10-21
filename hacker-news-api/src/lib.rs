//! A simple API client for the Hacker News firebase API.
mod client;
mod types;

pub use crate::client::{subscribe_top_stories, ApiClient};
pub use types::{Item, ResultExt, User};
