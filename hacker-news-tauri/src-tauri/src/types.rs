//! View model types.
use chrono::{DateTime, Utc};
use hacker_news_api::{Item, User};
use http_sanitizer::sanitize_html;
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
    pub position_change: PositionChange,
    pub ty: String,
}

impl From<Item> for HNItem {
    fn from(item: Item) -> Self {
        Self {
            id: item.id,
            kids: item.kids,
            text: item.text.map(sanitize_html),
            url: item.url,
            score: item.score,
            time: parse_date(item.time),
            by: item.by,
            has_rust: item.title.as_deref().map(has_rust).unwrap_or(false),
            title: item.title,
            viewed: false,
            new: false,
            position_change: PositionChange::UnChanged,
            ty: item.ty,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct HNUser {
    pub about: Option<String>,
    pub created: String,
    pub karma: u64,
}

impl From<User> for HNUser {
    fn from(user: User) -> Self {
        Self {
            about: user.about,
            created: DateTime::<Utc>::from_timestamp(user.created as i64, 0)
                .map(|d| format!("{}", d.format("%b %e, %Y")))
                .unwrap_or_default(),
            karma: user.karma,
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TopStories {
    pub items: Vec<HNItem>,
    pub loaded: String,
    pub rust_articles: usize,
    pub total_stories: usize,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum PositionChange {
    Up,
    Down,
    UnChanged,
}

/// Extract the duration from a UNIX time and convert duration into a human
/// friendly sentence.
fn parse_date(time: u64) -> Option<String> {
    let duration = DateTime::<Utc>::from_timestamp(time as i64, 0).map(|then| Utc::now() - then)?;

    let hours = duration.num_hours();
    let minutes = duration.num_minutes();
    let days = duration.num_days();

    match (days, hours, minutes) {
        (0, 0, 1) => "1 minute ago".to_string(),
        (0, 0, m) => format!("{m} minutes ago"),
        (0, 1, _) => "1 hour ago".to_string(),
        (0, h, _) => format!("{h} hours ago"),
        (1, _, _) => "1 day ago".to_string(),
        (d, _, _) => format!("{d} days ago"),
    }
    .into()
}

pub(crate) fn has_rust(title: &str) -> bool {
    title
        .to_lowercase()
        .split_ascii_whitespace()
        .any(|word| word.starts_with("rust"))
}
