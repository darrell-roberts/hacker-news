use crate::{has_rust, parse_date};
use hacker_news_api::Item;
use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HNItem {
    pub id: u64,
    pub kids: Vec<u64>,
    pub text: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub score: u64,
    pub time: Option<String>,
    pub by: String,
    pub has_rust: bool,
    pub viewed: bool,
    pub new: bool,
}

impl From<Item> for HNItem {
    fn from(item: Item) -> Self {
        Self {
            id: item.id,
            kids: item.kids,
            text: item.text,
            url: item.url,
            score: item.score,
            time: parse_date(item.time),
            by: item.by,
            has_rust: item.title.as_deref().map(has_rust).unwrap_or(false),
            title: item.title,
            viewed: false,
            new: false,
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TopStories {
    pub items: Vec<HNItem>,
    pub loaded: String,
    pub rust_articles: usize,
}
