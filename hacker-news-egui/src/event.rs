use anyhow::Result;
use egui::Context;
use hacker_news_api::{ApiClient, Item};
use log::error;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub enum Event {
    TopStories(Vec<Item>),
    Comments(Vec<Item>, Option<Item>),
    Back,
}

pub enum ClientEvent {
    TopStories,
    Comments(Vec<u64>, Option<Item>),
}

pub struct EventHandler {
    sender: UnboundedSender<ClientEvent>,
    client_receiver: UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(
        sender: UnboundedSender<ClientEvent>,
        client_receiver: UnboundedReceiver<Event>,
    ) -> Self {
        Self {
            sender,
            client_receiver,
        }
    }

    pub fn emit(&self, event: ClientEvent) -> Result<()> {
        Ok(self.sender.send(event)?)
    }

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
    pub fn new(client: Arc<ApiClient>, context: Context, sender: UnboundedSender<Event>) -> Self {
        Self {
            client,
            context,
            sender,
        }
    }

    pub async fn handle_event(&self, event: ClientEvent) {
        let result = match event {
            ClientEvent::TopStories => self
                .client
                .top_stories(50)
                .await
                .map_err(|e| anyhow::Error::msg(format!("{e}")))
                .and_then(|ts| {
                    self.sender
                        .send(Event::TopStories(ts))
                        .map_err(anyhow::Error::new)
                }),
            ClientEvent::Comments(ids, parent) => {
                self.client.items(&ids).await.and_then(|comments| {
                    self.sender
                        .send(Event::Comments(comments, parent))
                        .map_err(anyhow::Error::new)
                })
            }
        };

        match result {
            Ok(_) => self.context.request_repaint(),
            Err(err) => error!("handle_event failed: {err}"),
        }
    }
}
