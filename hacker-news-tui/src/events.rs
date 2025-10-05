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

pub enum AppEvent {
    CrossTerm(event::Event),
    UpdateProgress(RebuildProgress),
}

pub struct EventHandler {
    sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<AppEvent>();

        let s = Self { sender, receiver };
        s.subscribe_to_crossterm();
        s
    }

    pub fn next(&self) -> Result<AppEvent, RecvError> {
        self.receiver.recv()
    }

    /// Keyboard and mouse events.
    fn subscribe_to_crossterm(&self) {
        let tx = self.sender.clone();
        thread::spawn(move || {
            loop {
                let event = event::read().expect("Can no longer receive terminal events");
                tx.send(AppEvent::CrossTerm(event))
                    .expect("App event receiver is gone");
            }
        });
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
