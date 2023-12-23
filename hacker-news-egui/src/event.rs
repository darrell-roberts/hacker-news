use crate::app::ArticleType;
use anyhow::Result;
use egui::{Context, Id};
use hacker_news_api::{ApiClient, Item, User};
use log::error;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// Background event.
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
    FetchArticles(ClientEvent),
    ShowItemText(Item),
    FilterVisited,
    ResetVisited,
}

/// Client event.
pub enum ClientEvent {
    TopStories(usize),
    BestStories(usize),
    NewStories(usize),
    Comments {
        ids: Vec<u64>,
        parent: Option<Item>,
        id: Id,
    },
    User(String),
}

pub struct EventHandler {
    sender: UnboundedSender<ClientEvent>,
    client_receiver: UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Create a new [`EventHandler`].
    pub fn new(
        sender: UnboundedSender<ClientEvent>,
        client_receiver: UnboundedReceiver<Event>,
    ) -> Self {
        Self {
            sender,
            client_receiver,
        }
    }

    /// Emit a client event.
    pub fn emit(&self, event: ClientEvent) -> Result<()> {
        Ok(self.sender.send(event)?)
    }

    /// Get the next background event.
    pub fn next_event(&mut self) -> Result<Event> {
        Ok(self.client_receiver.try_recv()?)
    }
}

pub struct ClientEventHandler {
    client: Arc<ApiClient>,
    context: Context,
    sender: UnboundedSender<Event>,
}

impl ClientEventHandler {
    /// Create a new ['ClientEventHandler'].
    pub fn new(client: Arc<ApiClient>, context: Context, sender: UnboundedSender<Event>) -> Self {
        Self {
            client,
            context,
            sender,
        }
    }

    /// Handle a client event.
    pub async fn handle_event(&self, event: ClientEvent) {
        let result = match event {
            ClientEvent::TopStories(total) => match self.client.top_stories(total).await {
                Ok(ts) => self.sender.send(Event::Articles(ArticleType::Top, ts)),
                Err(err) => self.sender.send(Event::Error(err.to_string())),
            },
            ClientEvent::Comments { ids, parent, id } => match self.client.items(&ids).await {
                Ok(items) => self.sender.send(Event::Comments { items, parent, id }),
                Err(err) => self.sender.send(Event::Error(err.to_string())),
            },
            ClientEvent::BestStories(total) => match self.client.best_stories(total).await {
                Ok(bs) => self.sender.send(Event::Articles(ArticleType::Best, bs)),
                Err(err) => self.sender.send(Event::Error(err.to_string())),
            },
            ClientEvent::NewStories(total) => match self.client.new_stories(total).await {
                Ok(ns) => self.sender.send(Event::Articles(ArticleType::New, ns)),
                Err(err) => self.sender.send(Event::Error(err.to_string())),
            },
            ClientEvent::User(user) => match self.client.user(&user).await {
                Ok(user) => self.sender.send(Event::User(user)),
                Err(err) => self.sender.send(Event::Error(err.to_string())),
            },
        };

        self.context.request_repaint();
        if let Err(err) = result {
            error!("handle_event failed: {err}")
        }
    }
}
