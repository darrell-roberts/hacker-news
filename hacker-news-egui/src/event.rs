use crate::app::Filter;
use anyhow::Result;
use egui::Id;
use hacker_news_api::{ApiClient, ArticleType, Item, User};
use log::error;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// Client Event.
pub enum Event {
    Articles {
        ty: ArticleType,
        items: Vec<Item>,
        requested: usize,
    },
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
    FetchArticles {
        article_type: ArticleType,
        total: usize,
    },
    ShowItemText(Item),
    ToggleFilter(Filter),
    ResetVisited,
    ToggleOpenSearch,
    ZoomIn,
    ZoomOut,
}

/// API Event.
pub enum ApiEvent {
    Articles {
        ty: ArticleType,
        limit: usize,
    },
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
    client: ApiClient,
    sender: UnboundedSender<Event>,
}

impl ApiEventHandler {
    /// Create a new ['ApiEventHandler'].
    pub fn new(client: ApiClient, sender: UnboundedSender<Event>) -> Self {
        Self { client, sender }
    }

    /// Handle an api event.
    pub async fn handle_event(&self, event: ApiEvent) {
        let result = match event {
            ApiEvent::Articles { ty, limit } => match self.client.articles(limit, ty).await {
                Ok(items) => self.sender.send(Event::Articles {
                    ty,
                    items,
                    requested: limit,
                }),
                Err(err) => self.sender.send(Event::Error(err.to_string())),
            },
            ApiEvent::Comments { ids, parent, id } => match self.client.items(&ids).await {
                Ok(items) => self.sender.send(Event::Comments { items, parent, id }),
                Err(err) => self.sender.send(Event::Error(err.to_string())),
            },
            ApiEvent::User(user) => match self.client.user(&user).await {
                Ok(user) => self.sender.send(Event::User(user)),
                Err(err) => self.sender.send(Event::Error(err.to_string())),
            },
        };

        if let Err(err) = result {
            error!("handle_event failed: {err}")
        }
    }
}
