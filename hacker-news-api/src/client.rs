//! Hacker News API Client.
//!
//! I really want to use http/2 with multiplexing but I could not
//! figure out how to get that to work with firebase. From what I could
//! find out, the firebase realtime database only supports http/1 with
//! server side events.
//!
//! This is unfortunately just using the REST API with http/1 and due to
//! the structure of the REST API, multiple requests and connections are
//! required to do most operations.
use crate::{
    types::{Item, ItemEventData, ResultExt, TopStoriesEventData, User},
    ArticleType,
};
use anyhow::{Context, Result};
use futures::{future, stream::FuturesOrdered, TryFutureExt, TryStream, TryStreamExt};
use log::{error, info};
use reqwest::{header, IntoUrl};
use serde::Deserialize;
use std::{
    future::Future,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};
#[cfg(feature = "trace")]
use tracing::instrument;

pub struct Resolver {
    ip: Vec<SocketAddr>,
}

impl Resolver {
    fn new(host: &str) -> Result<Self> {
        Ok(Self {
            ip: host.to_socket_addrs()?.collect(),
        })
    }
}

impl reqwest::dns::Resolve for Resolver {
    fn resolve(&self, _name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let iter: Box<dyn Iterator<Item = SocketAddr> + Send> =
            Box::new(self.ip.clone().into_iter());
        Box::pin(async { Ok(iter) })
    }
}

/// Hacker News Api client.
pub struct ApiClient {
    client: reqwest::Client,
}

impl ApiClient {
    const API_END_POINT: &'static str = "https://hacker-news.firebaseio.com/v0";

    /// Create a new API client.
    pub fn new() -> Result<Self> {
        let resolver = Arc::new(Resolver::new("hacker-news.firebaseio.com:443")?);

        Ok(Self {
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .gzip(true)
                .dns_resolver(resolver)
                .tcp_keepalive(Duration::from_secs(60))
                .pool_max_idle_per_host(10)
                .use_rustls_tls()
                .no_proxy()
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
            // .inspect_err(|err| {
            //     dbg!(err);
            // })
            // .inspect_ok(|result| {
            //     info!("Using http version {:?}", result.version());
            //     info!("Headers: {:?}", result.headers())
            // })
            .and_then(|resp| resp.json::<Vec<u64>>())
            .await?;

        ids.truncate(limit);
        self.items(&ids).try_collect().await
    }

    #[cfg_attr(feature = "trace", instrument(skip_all))]
    pub fn articles(
        &self,
        limit: usize,
        article_type: ArticleType,
    ) -> impl Future<Output = Result<Vec<Item>>> + use<'_> {
        match article_type {
            ArticleType::New => self.call(limit, "newstories.json"),
            ArticleType::Best => self.call(limit, "beststories.json"),
            ArticleType::Top => self.call(limit, "topstories.json"),
            ArticleType::Ask => self.call(limit, "askstories.json"),
            ArticleType::Show => self.call(limit, "showstories.json"),
            ArticleType::Job => self.call(limit, "jobstories.json"),
        }
    }

    /// Get a single item via item id.
    #[cfg_attr(feature = "trace", instrument(skip_all))]
    pub fn item(&self, id: u64) -> impl Future<Output = Result<Item>> + use<'_> {
        self.client
            .get(format!("{}/item/{id}.json", Self::API_END_POINT,))
            .send()
            .and_then(|result| result.json::<Item>())
            .map_err(anyhow::Error::new)
    }

    /// Get multiple ids by item id.
    #[cfg_attr(feature = "trace", instrument(skip_all))]
    pub fn items(&self, ids: &[u64]) -> impl TryStream<Ok = Item, Error = anyhow::Error> {
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
            .map_err(anyhow::Error::new)
            .into_stream()
    }

    /// Get user by user handle.
    pub fn user(&self, handle: &str) -> impl Future<Output = Result<User>> + use<'_> {
        self.client
            .get(format!("{}/user/{handle}.json", Self::API_END_POINT))
            .send()
            .and_then(|resp| resp.json::<User>())
            .map_err(anyhow::Error::new)
    }

    /// Subscribe to a a server side event and return a stream that yields the generic
    /// event data type.
    fn event_source<EventData>(
        &self,
        url: impl IntoUrl,
    ) -> impl Future<
        Output = reqwest::Result<impl TryStream<Ok = Option<EventData>, Error = anyhow::Error>>,
    >
    where
        EventData: for<'a> Deserialize<'a>,
    {
        self.client
            .get(url)
            .header(header::ACCEPT, "text/event-stream")
            .send()
            .map_ok(|response| {
                response
                    .bytes_stream()
                    .map_ok(|bytes| parse_event(&bytes))
                    .map_err(anyhow::Error::new)
            })
    }

    /// Top stories event-source stream.
    pub async fn top_stories_stream(&self, sender: Sender<TopStoriesEventData>) -> Result<()> {
        let mut stream = self
            .event_source::<TopStoriesEventData>(format!("{}/topstories.json", Self::API_END_POINT))
            .await?;

        while let Some(event) = stream.try_next().await? {
            if let Some(data) = event {
                sender.send(data).await?;
            }
        }
        Ok(())
    }

    /// Subscribe to updates to a story via server side event sourcing.
    pub async fn item_stream(&self, story_id: u64, sender: Sender<ItemEventData>) -> Result<()> {
        let mut stream = self
            .event_source::<ItemEventData>(format!("{}/item/{story_id}.json", Self::API_END_POINT))
            .await?;

        while let Some(event) = stream.try_next().await? {
            if let Some(data) = event {
                sender.send(data).await?;
            }
        }
        info!("item stream has exited.");
        Ok(())
    }
}

/// Parse an event from the event-source.
fn parse_event<EventData>(bytes: &[u8]) -> Option<EventData>
where
    EventData: for<'a> Deserialize<'a>,
{
    let mut lines = bytes.split(|b| *b == b'\n');
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
pub fn subscribe_top_stories() -> (Receiver<TopStoriesEventData>, JoinHandle<()>) {
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
