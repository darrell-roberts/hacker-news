//! Application state and type definitions.
use crate::{
    event::{ClientEvent, Event, EventHandler},
    renderer, SHUT_DOWN,
};
use chrono::{DateTime, Local};
use eframe::Storage;
use egui::{os::OperatingSystem, Id};
use hacker_news_api::{Item, ResultExt, User};
use std::{collections::HashSet, str::FromStr, sync::atomic::Ordering};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum ArticleType {
    New,
    Best,
    Top,
    Ask,
    Show,
    Job,
}

impl ArticleType {
    pub fn as_str(&self) -> &str {
        match self {
            ArticleType::New => "New",
            ArticleType::Best => "Best",
            ArticleType::Top => "Top",
            ArticleType::Ask => "Ask",
            ArticleType::Show => "Show",
            ArticleType::Job => "Job",
        }
    }
}

impl FromStr for ArticleType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "New" => ArticleType::New,
            "Best" => ArticleType::Best,
            "Top" => ArticleType::Top,
            _ => return Err(()),
        })
    }
}

/// A list of child comment ids for a given comment.
pub struct CommentItem {
    /// Sub comments.
    pub comments: Vec<Item>,
    /// parent of sub comment
    pub parent: Option<Item>,
    /// Id for widget uniqueness. Can be
    /// the article item if this is a top level
    /// comment otherwise the parent comment id.
    pub id: Id,
}

/// Comment state data.
#[derive(Default)]
pub struct CommentsState {
    /// Active comments being viewed.
    pub comments: Vec<Item>,
    /// Trail of comments navigated.
    pub comment_trail: Vec<CommentItem>,
    /// Parent comment trail.
    pub parent_comments: Vec<Item>,
    /// Active item when reading comments.
    pub active_item: Option<Item>,
}

/// Article filters.
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum Filter {
    Jobs,
    Stories,
    Polls,
    Visisted,
}

/// Application State.
pub struct HackerNewsApp {
    /// Top stories.
    pub articles: Vec<Item>,
    /// Event handler for background events.
    event_handler: EventHandler,
    /// API request in progress.
    pub fetching: bool,
    /// Emit local events.
    pub local_sender: UnboundedSender<Event>,
    /// Number of articles to show.
    pub showing: usize,
    /// Articles visited.
    pub visited: Vec<u64>,
    /// Comments state.
    pub comments_state: CommentsState,
    /// Errors.
    pub error: Option<String>,
    /// Viewing article type.
    pub article_type: ArticleType,
    /// Comment window open states.
    pub viewing_comments: Vec<bool>,
    /// Viewing a user
    pub user: Option<User>,
    /// User window open/closed.
    pub viewing_user: bool,
    /// Search input.
    pub search: String,
    /// Showing window for item text.
    pub viewing_item_text: bool,
    /// Filters.
    pub filters: HashSet<Filter>,
    /// Last update of articles.
    pub last_update: Option<DateTime<Local>>,
    /// Search input is open.
    pub search_open: bool,
}

/// State that requires mutation by a widget. This is the
/// case for text edit and windows which take a `&mut String`
/// or `&mut bool`
pub struct MutableWidgetState {
    pub search: String,
    pub viewing_comments: Vec<bool>,
    pub viewing_user: bool,
    pub viewing_item_text: bool,
}

impl HackerNewsApp {
    /// Create a new [`HackerNewsApp`].
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        event_handler: EventHandler,
        local_sender: UnboundedSender<Event>,
    ) -> Self {
        let visited = cc
            .storage
            .and_then(|s| s.get_string("visited"))
            .map(|v| {
                v.split(',')
                    .flat_map(|n| n.parse::<u64>())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let article_type = cc
            .storage
            .and_then(|s| s.get_string("article_type"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(ArticleType::Top);

        let showing = cc
            .storage
            .and_then(|s| s.get_string("showing"))
            .and_then(|showing| showing.parse().ok())
            .unwrap_or(50);

        Self {
            event_handler,
            articles: Vec::new(),
            fetching: true,
            local_sender,
            showing,
            visited,
            comments_state: Default::default(),
            error: None,
            article_type,
            viewing_comments: Vec::new(),
            user: None,
            viewing_user: false,
            search: String::new(),
            viewing_item_text: false,
            filters: HashSet::new(),
            last_update: None,
            search_open: false,
        }
    }

    /// Handle emitted events.
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Articles(article_type, ts) => {
                self.showing = ts.len();
                self.articles = ts;
                self.error = None;
                self.article_type = article_type;
                self.fetching = false;
                self.last_update = Some(Local::now())
            }
            Event::Comments { items, parent, id } => {
                let comment_item = CommentItem {
                    comments: items,
                    parent,
                    id,
                };
                if comment_item.parent.is_some() {
                    self.comments_state.comment_trail.push(comment_item);
                    self.viewing_comments.push(true);
                } else {
                    // Reset comment history/state.
                    self.comments_state.comment_trail = vec![comment_item];
                    self.viewing_comments = vec![true];
                }
                self.error = None;
                self.fetching = false;
            }

            Event::Error(error) => {
                self.fetching = false;
                self.error = Some(error);
            }
            Event::User(user) => {
                self.fetching = false;
                self.viewing_user = true;
                self.user = Some(user);
            }
            Event::FetchUser(user) => {
                self.fetching = true;
                self.event_handler
                    .emit(ClientEvent::User(user))
                    .log_error_consume();
            }
            Event::FetchComments {
                ids,
                parent,
                id,
                active_item,
            } => {
                self.fetching = true;
                if let Some(item) = active_item {
                    // Top level comment.
                    self.comments_state.comments = Vec::new();
                    self.visited.push(item.id);
                    self.comments_state.active_item = Some(item);
                }
                self.event_handler
                    .emit(ClientEvent::Comments { ids, parent, id })
                    .log_error_consume();
            }
            Event::Visited(id) => {
                self.visited.push(id);
            }
            Event::FetchArticles(event) => {
                self.fetching = true;
                self.event_handler.emit(event).log_error_consume();
            }
            Event::ShowItemText(item) => {
                self.visited.push(item.id);
                self.comments_state.active_item = Some(item);
                self.viewing_item_text = true;
            }
            Event::ToggleFilter(filter) => {
                let removed = self.filters.remove(&filter);
                if !removed {
                    self.filters.insert(filter);
                }
            }
            Event::ResetVisited => {
                self.visited.clear();
            }
            Event::ToggleOpenSearch => {
                self.search_open = !self.search_open;
            }
        }
    }

    pub fn last_request(&self) -> impl Fn(usize) -> ClientEvent {
        match self.article_type {
            ArticleType::New => ClientEvent::NewStories,
            ArticleType::Best => ClientEvent::BestStories,
            ArticleType::Top => ClientEvent::TopStories,
            ArticleType::Ask => ClientEvent::AskStories,
            ArticleType::Show => ClientEvent::ShowStories,
            ArticleType::Job => ClientEvent::JobStories,
        }
    }

    /// Handle background emitted events.
    fn handle_next_event(&mut self) {
        self.event_handler
            .next_event()
            .map(|event| self.handle_event(event))
            .unwrap_or_default();
    }
}

impl eframe::App for HackerNewsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_next_event();

        if ctx.os() == OperatingSystem::Mac {
            ctx.set_pixels_per_point(2.5);
        } else {
            ctx.set_pixels_per_point(3.0);
        }

        // I would prefer having Render not mutate state however
        // Window widget requires a mutable reference for the close
        // button in the title bar and the search input also uses
        // a mutable ref for the input String.
        let mut mutable_state = MutableWidgetState {
            viewing_comments: self.viewing_comments.clone(),
            search: self.search.clone(),
            viewing_user: self.viewing_user,
            viewing_item_text: self.viewing_item_text,
        };

        renderer::render(ctx, self, &mut mutable_state);

        if mutable_state.viewing_comments != self.viewing_comments {
            self.viewing_comments = mutable_state.viewing_comments
        }
        if mutable_state.search != self.search {
            self.search = mutable_state.search;
        }
        self.viewing_user = mutable_state.viewing_user;
        self.viewing_item_text = mutable_state.viewing_item_text;
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        SHUT_DOWN.store(true, Ordering::Release);
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        storage.set_string("visited", {
            let strs = self
                .visited
                .iter()
                .map(|id| format!("{id}"))
                .collect::<Vec<_>>();
            strs.join(",")
        });
        storage.set_string("showing", format!("{}", self.showing));
        storage.set_string("article_type", self.article_type.as_str().into());
    }
}
