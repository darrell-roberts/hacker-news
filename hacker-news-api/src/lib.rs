use futures::TryFutureExt;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{self, Receiver, Sender};

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
                    // .timeout(Duration::from_secs(30))
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

    pub async fn top_stories_stream(&self, sender: Sender<EventData>) -> Result<()> {
        use futures::stream::StreamExt;
        let mut stream = self
            .client
            .get(format!("{}/topstories.json", Self::API_END_POINT))
            .header("Accept", "text/event-stream")
            .send()
            .await?
            .bytes_stream();

        while let Some(item) = stream.next().await {
            let bytes = item?;

            if let Some(data) = parse_event(&bytes) {
                sender.send(data).await?;
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct EventData {
    pub path: String,
    pub data: Vec<u64>,
}

fn parse_event(bytes: &[u8]) -> Option<EventData> {
    let mut lines = bytes.split(|b| *b == b'\n');

    if let Some(event) = lines.next() {
        if event.starts_with(b"event: ") {
            let event_name = String::from_utf8_lossy(&event[7..]);
            info!("event_name: {event_name}");
            if event_name != "put" {
                return None;
            }
        }
    }

    if let Some(data) = lines.next() {
        if data.starts_with(b"data: ") {
            let event_data = serde_json::from_slice::<EventData>(&data[6..])
                .map_err(|err| {
                    error!("Failed to deserialize event data {err}");
                    err
                })
                .ok()?;
            return Some(event_data);
        }
    }
    None
}

pub fn subscribe_top_stories() -> Receiver<EventData> {
    let (tx, rx) = mpsc::channel(100);

    let _ = tokio::spawn(async move {
        loop {
            match ApiClient::new() {
                Ok(client) => {
                    if let Err(err) = client.top_stories_stream(tx.clone()).await {
                        error!("Event stream severed {err}");
                    }
                }
                Err(err) => {
                    error!("Failed to create client {err}");
                }
            }
            tokio::time::sleep(Duration::from_secs(60 * 5)).await;
            info!("Restarted subscription");
        }
    });

    rx
}
