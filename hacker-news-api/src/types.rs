//! API Client types.
use log::error;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

/// Hacker news item.
///
/// [`https://github.com/HackerNews/API`]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    /// The item's unique id.
    pub id: u64,
    #[serde(default)]
    pub kids: Vec<u64>,
    /// The comment, story or poll text. HTML.
    pub text: Option<String>,
    /// The URL of the story.
    pub url: Option<String>,
    /// The title of the story, poll or job. HTML.
    pub title: Option<String>,
    #[serde(default)]
    pub score: u64,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
    #[serde(default)]
    /// The username of the item's author.
    pub by: String,
    #[serde(default)]
    /// true if the item is dead.
    pub dead: bool,
    #[serde(default)]
    /// true if the item is deleted.
    pub deleted: bool,
    #[serde(alias = "type")]
    /// The type of item. One of "job", "story", "comment", "poll", or "pollopt".
    pub ty: String,
    /// The comment's parent: either another comment or the relevant story.
    pub parent: Option<u64>,
    /// In the case of stories or polls, the total comment count.
    pub descendants: Option<u64>,
}

/// Hacker news user.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub about: Option<String>,
    pub created: u64,
    pub karma: u64,
    #[serde(default)]
    pub submitted: Vec<u64>,
}

/// An event-source for hacker-news.
#[derive(Deserialize, Debug)]
pub struct StoriesEventData {
    pub path: String,
    pub data: Vec<u64>,
}

#[derive(Deserialize, Debug)]
pub struct ItemEventData {
    pub path: String,
    pub data: Item,
}

/// Extension trait for the Result type to add logging capabilities.
pub trait ResultExt<T, E> {
    /// If the result is [`Err`] then log the error.
    fn log_error(self) -> Self;

    /// When you don't need the result but want to log failure.
    fn log_error_consume(self);
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

    fn log_error_consume(self) {
        let _ = self.log_error();
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Serialize, Deserialize, Default)]
pub enum ArticleType {
    New,
    Best,
    #[default]
    Top,
    Ask,
    Show,
    Job,
}

impl ArticleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArticleType::New => "New",
            ArticleType::Best => "Best",
            ArticleType::Top => "Top",
            ArticleType::Ask => "Ask",
            ArticleType::Show => "Show",
            ArticleType::Job => "Job",
        }
    }
}

impl FromStr for ArticleType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "New" => ArticleType::New,
            "Best" => ArticleType::Best,
            "Top" => ArticleType::Top,
            "Ask" => ArticleType::Ask,
            "Show" => ArticleType::Show,
            "Job" => ArticleType::Job,
            _ => return Err(()),
        })
    }
}

impl Display for ArticleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
