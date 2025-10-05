//! Background events
use std::{
    sync::{
        Arc, RwLock,
        mpsc::{Receiver, RecvError, Sender, channel},
    },
    thread,
};

use crossterm::event;
use futures::StreamExt as _;
use hacker_news_api::ArticleType;
use hacker_news_search::{RebuildProgress, SearchContext};

#[derive(Debug)]
pub struct IndexRebuildState {
    /// Total number of items that will be indexed.
    pub total_items: f64,
    /// The current number of items that have been indexed.
    pub total_rebuilt: f64,
}

impl IndexRebuildState {
    /// Rebuild status as completion percentage.
    pub fn percent(&self) -> u16 {
        ((self.total_rebuilt / self.total_items) * 100.) as u16
    }
}

/// Background application event
pub enum AppEvent {
    /// Keyboard or mouse event.
    CrossTerm(event::Event),
    /// Index rebuild progress event.
    UpdateProgress(RebuildProgress),
}

/// Event manager.
pub struct EventManager {
    sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,
}

impl EventManager {
    /// Create an event manager and subscribe to crossterm events.
    pub fn new() -> Self {
        let (sender, receiver) = channel::<AppEvent>();
        Self { sender, receiver }.subscribe_to_crossterm()
    }

    /// Wait for the next event.
    pub fn next(&self) -> Result<AppEvent, RecvError> {
        self.receiver.recv()
    }

    /// Keyboard and mouse events.
    fn subscribe_to_crossterm(self) -> Self {
        let tx = self.sender.clone();
        thread::spawn(move || {
            loop {
                let event = event::read().expect("Can no longer receive terminal events");
                tx.send(AppEvent::CrossTerm(event))
                    .expect("App event receiver is gone");
            }
        });
        self
    }

    /// Spawn a tokio task that will emit rebuild index events
    pub fn rebuild_index(&self, search_context: Arc<RwLock<SearchContext>>) {
        let (tx, mut rx) = futures::channel::mpsc::channel::<RebuildProgress>(100);

        let sender = self.sender.clone();
        tokio::spawn(async move {
            while let Some(status) = rx.next().await {
                sender.send(AppEvent::UpdateProgress(status)).unwrap();
            }
        });

        let fut = hacker_news_search::rebuild_index(search_context, ArticleType::Top, tx);
        tokio::spawn(fut);
    }
}
