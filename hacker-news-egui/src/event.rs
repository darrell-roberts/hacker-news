use crate::app::Filter;
use anyhow::Result;
use egui::{Context, Id};
use hacker_news_api::{ApiClient, ArticleType, Item, User};
use log::error;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// Client Event.
pub enum Event {
    Articles(ArticleType, Vec<Item>),
    Comments {
        items: Vec<Item>,
        parent: Option<Item>,
        id: Id,
    },
    Error(String),
    User(User),
    FetchUser(String),
    FetchComments {
        ids: Vec<u64>,
        parent: Option<Item>,
        id: Id,
        active_item: Option<Item>,
    },
    Visited(u64),
    FetchArticles(ApiEvent),
    ShowItemText(Item),
    ToggleFilter(Filter),
    ResetVisited,
    ToggleOpenSearch,
}

/// API Event.
pub enum ApiEvent {
    TopStories(usize),
    BestStories(usize),
    NewStories(usize),
    AskStories(usize),
    ShowStories(usize),
    JobStories(usize),
    Comments {
        ids: Vec<u64>,
        parent: Option<Item>,
        id: Id,
    },
    User(String),
}

pub struct EventHandler {
    sender: UnboundedSender<ApiEvent>,
    client_receiver: UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Create a new [`EventHandler`].
    pub fn new(
        sender: UnboundedSender<ApiEvent>,
        client_receiver: UnboundedReceiver<Event>,
    ) -> Self {
        Self {
            sender,
            client_receiver,
        }
    }

    /// Emit a client event.
    pub fn emit(&self, event: ApiEvent) -> Result<()> {
        Ok(self.sender.send(event)?)
    }

    /// Get the next background event.
    pub fn next_event(&mut self) -> Result<Event> {
        Ok(self.client_receiver.try_recv()?)
    }
}

pub struct ApiEventHandler {
    client: Arc<ApiClient>,
    context: Context,
    sender: UnboundedSender<Event>,
}

impl ApiEventHandler {
    /// Create a new ['ApiEventHandler'].
    pub fn new(client: Arc<ApiClient>, context: Context, sender: UnboundedSender<Event>) -> Self {
        Self {
            client,
            context,
            sender,
        }
    }

    /// Handle an api event.
    pub async fn handle_event(&self, event: ApiEvent) {
        let result = match event {
            ApiEvent::TopStories(total) => {
                match self.client.articles(total, ArticleType::Top).await {
                    Ok(ts) => self.sender.send(Event::Articles(ArticleType::Top, ts)),
                    Err(err) => self.sender.send(Event::Error(err.to_string())),
                }
            }
            ApiEvent::Comments { ids, parent, id } => match self.client.items(&ids).await {
                Ok(items) => self.sender.send(Event::Comments { items, parent, id }),
                Err(err) => self.sender.send(Event::Error(err.to_string())),
            },
            ApiEvent::BestStories(total) => {
                match self.client.articles(total, ArticleType::Best).await {
                    Ok(bs) => self.sender.send(Event::Articles(ArticleType::Best, bs)),
                    Err(err) => self.sender.send(Event::Error(err.to_string())),
                }
            }
            ApiEvent::NewStories(total) => {
                match self.client.articles(total, ArticleType::New).await {
                    Ok(ns) => self.sender.send(Event::Articles(ArticleType::New, ns)),
                    Err(err) => self.sender.send(Event::Error(err.to_string())),
                }
            }
            ApiEvent::User(user) => match self.client.user(&user).await {
                Ok(user) => self.sender.send(Event::User(user)),
                Err(err) => self.sender.send(Event::Error(err.to_string())),
            },
            ApiEvent::AskStories(total) => {
                match self.client.articles(total, ArticleType::Ask).await {
                    Ok(items) => self.sender.send(Event::Articles(ArticleType::Ask, items)),
                    Err(err) => self.sender.send(Event::Error(err.to_string())),
                }
            }
            ApiEvent::ShowStories(total) => {
                match self.client.articles(total, ArticleType::Show).await {
                    Ok(items) => self.sender.send(Event::Articles(ArticleType::Show, items)),
                    Err(err) => self.sender.send(Event::Error(err.to_string())),
                }
            }
            ApiEvent::JobStories(total) => {
                match self.client.articles(total, ArticleType::Job).await {
                    Ok(items) => self.sender.send(Event::Articles(ArticleType::Job, items)),
                    Err(err) => self.sender.send(Event::Error(err.to_string())),
                }
            }
        };

        self.context.request_repaint();
        if let Err(err) = result {
            error!("handle_event failed: {err}")
        }
    }
}
