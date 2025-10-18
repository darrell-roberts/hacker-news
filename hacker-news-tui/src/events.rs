//! Background events
use futures::StreamExt as _;
use hacker_news_api::ArticleType;
use hacker_news_search::{IndexStats, RebuildProgress, SearchContext, api::Story, update_story};
use log::error;
use ratatui::crossterm::event;
use std::{
    sync::{
        Arc, RwLock,
        mpsc::{Receiver, RecvError, Sender, channel},
    },
    thread,
};

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
    /// Indexing completed
    IndexingCompleted(IndexStats),
    /// Story updated
    StoryUpdated(Story),
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
    pub fn rebuild_index(
        &self,
        search_context: Arc<RwLock<SearchContext>>,
        article_type: ArticleType,
    ) {
        let (tx, mut rx) = futures::channel::mpsc::channel::<RebuildProgress>(100);

        let sender = self.sender.clone();
        tokio::spawn(async move {
            while let Some(status) = rx.next().await {
                sender.send(AppEvent::UpdateProgress(status)).unwrap();
            }
        });

        tokio::spawn(rebuild(
            search_context,
            tx,
            self.sender.clone(),
            article_type,
        ));
    }

    /// Update a single story.
    pub fn update_story(&self, search_context: Arc<RwLock<SearchContext>>, story: Story) {
        let tx = self.sender.clone();
        tokio::spawn(async move {
            let result = update_story(search_context, story).await;
            match result {
                Ok(story) => {
                    if let Some(story) = story {
                        tx.send(AppEvent::StoryUpdated(story)).unwrap();
                    }
                }
                Err(err) => {
                    error!("Failed to update story: {err}");
                }
            }
        });
    }
}

async fn rebuild(
    search_context: Arc<RwLock<SearchContext>>,
    tx_progress: futures::channel::mpsc::Sender<RebuildProgress>,
    tx_result: Sender<AppEvent>,
    article_type: ArticleType,
) {
    let stats = hacker_news_search::rebuild_index(search_context, article_type, tx_progress).await;
    match stats {
        Ok(stats) => {
            tx_result.send(AppEvent::IndexingCompleted(stats)).unwrap();
        }
        Err(err) => {
            error!("Failed to build index: {err}");
        }
    }
}
