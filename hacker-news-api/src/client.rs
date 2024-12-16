//! Hacker News API Client.
use crate::{
    types::{EventData, Item, ResultExt, User},
    ArticleType,
};
use anyhow::{Context, Result};
use futures::{
    future,
    stream::{FuturesOrdered, FuturesUnordered},
    Stream, TryFutureExt, TryStreamExt,
};
use log::{error, info};
use std::time::Duration;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};
#[cfg(feature = "trace")]
use tracing::instrument;

/// Hacker News Api client.
pub struct ApiClient {
    client: reqwest::Client,
}

impl ApiClient {
    const API_END_POINT: &'static str = "https://hacker-news.firebaseio.com/v0";

    /// Create a new API client.
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .gzip(true)
                .hickory_dns(true)
                .pool_max_idle_per_host(100)
                // .http2_prior_knowledge()
                // .pool_max_idle_per_host(1)
                .build()
                .context("Failed to create api client")?,
        })
    }

    /// Make firebase api call.
    #[cfg_attr(feature = "trace", instrument(skip_all))]
    async fn call(&self, limit: usize, api: &str) -> Result<Vec<Item>> {
        let mut ids = self
            .client
            .get(format!("{}/{api}", Self::API_END_POINT))
            .send()
            .and_then(|resp| resp.json::<Vec<u64>>())
            .await?;

        ids.truncate(limit);
        self.items(&ids).await
    }

    #[cfg_attr(feature = "trace", instrument(skip_all))]
    pub async fn articles(&self, limit: usize, article_type: ArticleType) -> Result<Vec<Item>> {
        match article_type {
            ArticleType::New => self.call(limit, "newstories.json").await,
            ArticleType::Best => self.call(limit, "beststories.json").await,
            ArticleType::Top => self.call(limit, "topstories.json").await,
            ArticleType::Ask => self.call(limit, "askstories.json").await,
            ArticleType::Show => self.call(limit, "showstories.json").await,
            ArticleType::Job => self.call(limit, "jobstories.json").await,
        }
    }

    /// Get a single item via item id.
    #[cfg_attr(feature = "trace", instrument(skip_all))]
    pub async fn item(&self, id: u64) -> Result<Item> {
        self.client
            .get(format!("{}/item/{id}.json", Self::API_END_POINT,))
            .send()
            .await
            .context("Failed to send request")?
            .json::<Item>()
            .await
            .context("Failed to deserialize item")
    }

    /// Get multiple ids by item id.
    #[cfg_attr(feature = "trace", instrument(skip_all))]
    pub async fn items(&self, ids: &[u64]) -> Result<Vec<Item>> {
        // The firebase api only provides the option to get each item one by
        // one.
        ids.iter()
            .map(|id| {
                let client = &self.client;
                client
                    .get(format!("{}/item/{id}.json", Self::API_END_POINT,))
                    .send()
                    .and_then(|resp| resp.json::<Item>())
            })
            .collect::<FuturesOrdered<_>>()
            .try_filter(|item| future::ready(!(item.dead || item.deleted)))
            .try_collect::<Vec<_>>()
            .await
            .map_err(anyhow::Error::new)
    }

    /// Spawn a task that makes a call to each item endpoint and returns a stream
    /// of results.
    #[cfg_attr(feature = "trace", instrument(skip_all))]
    pub fn items_stream(&self, ids: &[u64]) -> JoinHandle<impl Stream<Item = Result<(u64, Item)>>> {
        // The firebase api only provides the option to get each item one by
        // one.
        let futures = ids
            .iter()
            .copied()
            .zip(1_u64..)
            .map(|(id, rank)| {
                let client = &self.client;
                client
                    .get(format!("{}/item/{id}.json", Self::API_END_POINT,))
                    .send()
                    .and_then(|resp| resp.json::<Item>())
                    .map_ok(move |item| (rank, item))
                    .map_err(anyhow::Error::new)
            })
            .collect::<FuturesUnordered<_>>();

        tokio::spawn(async {
            futures
                .into_stream()
                .try_filter(|item| future::ready(!(item.1.dead || item.1.deleted)))
        })
    }

    /// Get user by user handle.
    pub async fn user(&self, handle: &str) -> Result<User> {
        self.client
            .get(format!("{}/user/{handle}.json", Self::API_END_POINT))
            .send()
            .and_then(|resp| resp.json::<User>())
            .await
            .context("Failed to deserialize user")
    }

    /// Top stories event-source stream.
    pub async fn top_stories_stream(&self, sender: Sender<EventData>) -> Result<()> {
        let mut stream = self
            .client
            .get(format!("{}/topstories.json", Self::API_END_POINT))
            .header("Accept", "text/event-stream")
            .send()
            .await
            .context("Failed to send request")?
            .bytes_stream();

        while let Some(bytes) = stream.try_next().await? {
            if let Some(data) = parse_event(&bytes) {
                sender.send(data).await?;
            }
        }

        Ok(())
    }
}

/// Parse an event from the event-source.
fn parse_event(bytes: &[u8]) -> Option<EventData> {
    let mut lines = bytes.split(|b| *b == b'\n');

    // We are only concerned with put events for the top stories. This event
    // will provide a JSON number array payload of all the top stories in
    // ranking order.

    lines
        .next()?
        .starts_with(b"event: put")
        .then(|| lines.next())?
        .filter(|data| data.starts_with(b"data: "))
        .and_then(|data| {
            serde_json::from_slice::<EventData>(&data[6..])
                .with_context(|| {
                    format!(
                        "Failed to deserialize event payload: {}",
                        String::from_utf8_lossy(&data[6..])
                    )
                })
                .log_error()
                .ok()
        })
}

/// Create a subscription to the top stories event stream. Provides a receive
/// channel and a task handle for consuming events and canceling the task.
pub fn subscribe_top_stories() -> (Receiver<EventData>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel(100);

    let handle = tokio::spawn(async move {
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
            info!("Restarting subscription");
        }
    });

    (rx, handle)
}
