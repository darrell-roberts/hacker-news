use log::error;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

/// Hacker news item.
///
/// [`https://github.com/HackerNews/API`]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    pub id: u64,
    #[serde(default)]
    pub kids: Vec<u64>,
    pub text: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub score: u64,
    pub time: u64,
    #[serde(default)]
    pub by: String,
    #[serde(default)]
    pub dead: bool,
    #[serde(default)]
    pub deleted: bool,
    #[serde(alias = "type")]
    pub ty: String,
}

/// Hacker news user.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub about: Option<String>,
    pub created: u64,
    pub karma: u64,
    pub submitted: Vec<u64>,
}

/// An event-source for hacker-news.
#[derive(Deserialize, Debug)]
pub struct EventData {
    pub path: String,
    pub data: Vec<u64>,
}

/// Extension trait for the Result type.
pub trait ResultExt<T, E> {
    /// If the result is [`Err`] then log the error.
    fn log_error(self) -> Self;
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: Display,
{
    fn log_error(self) -> Self {
        match self {
            o @ Ok(_) => o,
            Err(err) => {
                error!("{err}");
                Err(err)
            }
        }
    }
}
