use crate::app::ArticleType;
use anyhow::Result;
use egui::Context;
use hacker_news_api::{ApiClient, Item};
use log::error;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// Background event.
pub enum Event {
    Articles(ArticleType, Vec<Item>),
    Comments(Vec<Item>, Option<Item>),
    Back,
    Error(String),
}

/// Client event.
pub enum ClientEvent {
    TopStories(usize),
    BestStories(usize),
    NewStories(usize),
    Comments(Vec<u64>, Option<Item>),
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
            ClientEvent::Comments(ids, parent) => match self.client.items(&ids).await {
                Ok(comments) => self.sender.send(Event::Comments(comments, parent)),
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
        };

        self.context.request_repaint();
        if let Err(err) = result {
            error!("handle_event failed: {err}")
        }
    }
}
