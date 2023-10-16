use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

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
}

/// Hacker News Api client.
pub struct ApiClient {
    client: Arc<reqwest::Client>,
}

type Result<T> = std::result::Result<T, anyhow::Error>;

impl ApiClient {
    const API_END_POINT: &str = "https://hacker-news.firebaseio.com/v0";

    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Arc::new(
                reqwest::Client::builder()
                    .timeout(Duration::from_secs(30))
                    .build()?,
            ),
        })
    }

    /// Get `limit` number of top stories.
    pub async fn top_stories(&self, limit: usize) -> Result<Vec<Item>> {
        let mut ids = self
            .client
            .get(format!("{}/topstories.json", Self::API_END_POINT))
            .send()
            .and_then(|resp| resp.json::<Vec<u64>>())
            .await?;

        ids.truncate(limit);
        let results = self.items(ids).await?;

        Ok(results)
    }

    /// Get a single item via item id.
    pub async fn item(&self, id: u64) -> Result<Item> {
        self.client
            .get(format!("{}/item/{id}.json", Self::API_END_POINT,))
            .send()
            .and_then(|resp| resp.json::<Item>())
            .map_err(anyhow::Error::new)
            .await
    }

    /// Get multiple ids by item id.
    pub async fn items(&self, ids: Vec<u64>) -> Result<Vec<Item>> {
        // The firebase api only provides the option to get each item one by
        // one.
        let mut handles = Vec::with_capacity(ids.len());
        for id in ids {
            let client = self.client.clone();
            handles.push(tokio::spawn(
                client
                    .get(format!("{}/item/{id}.json", Self::API_END_POINT,))
                    .send()
                    .and_then(|resp| resp.json::<Item>()),
            ));
        }

        let mut result = Vec::with_capacity(handles.len());

        for h in handles {
            let item = h.await??;
            if !(item.dead || item.deleted) {
                result.push(item);
            }
        }

        Ok(result)
    }
}
