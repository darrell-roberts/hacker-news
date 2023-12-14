use anyhow::Result;
use hacker_news_api::Item;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub static SHUT_DOWN: AtomicBool = AtomicBool::new(false);

pub enum Event {
    TopStories(Vec<Item>),
    Comments(Vec<Item>),
}

pub enum ClientEvent {
    TopStories,
    Comments(Vec<u64>),
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
