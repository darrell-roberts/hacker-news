//! App state, management and root widget.
use std::sync::{Arc, RwLock};

use crate::{
    articles::ArticlesWidget,
    events::{AppEvent, EventManager, IndexRebuildState},
    footer::FooterWidget,
};
use app_dirs2::{AppInfo, get_app_dir};
use color_eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};
use hacker_news_api::ArticleType;
use hacker_news_search::{RebuildProgress, SearchContext, api::Story, api_client};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};

/// The main application which holds the state and logic of the application.
pub struct App {
    event_manager: EventManager,
    /// Is the application running?
    running: bool,
    search_context: Arc<RwLock<SearchContext>>,
    top_stories: Vec<Story>,
    selected_item: Option<usize>,
    pub rebuild_progress: Option<IndexRebuildState>,
}

pub const APP_INFO: AppInfo = AppInfo {
    name: "Hacker News",
    author: "Somebody",
};

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> color_eyre::Result<Self> {
        let _ = api_client();

        let index_dir = get_app_dir(
            app_dirs2::AppDataType::UserData,
            &APP_INFO,
            "hacker-news-index",
        )?;
        let search_context = Arc::new(RwLock::new(SearchContext::new(
            &index_dir,
            ArticleType::Top,
        )?));

        let top_stories = search_context.read().unwrap().top_stories(75, 0)?;

        Ok(Self {
            event_manager: EventManager::new(),
            running: false,
            search_context,
            top_stories,
            selected_item: None,
            rebuild_progress: None,
        })
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_event(self.event_manager.next()?);
        }
        Ok(())
    }

    fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::CrossTerm(event) => self.handle_crossterm_event(event),
            AppEvent::UpdateProgress(rebuild_progress) => {
                self.handle_rebuild_progress(rebuild_progress)
            }
        }
    }

    fn handle_rebuild_progress(&mut self, progress: RebuildProgress) {
        match progress {
            RebuildProgress::Started(total_items) => {
                self.rebuild_progress = Some(IndexRebuildState {
                    total_items: total_items as f64,
                    total_rebuilt: 0.,
                });
            }
            RebuildProgress::StoryCompleted => {
                if let Some(state) = self.rebuild_progress.as_mut() {
                    state.total_rebuilt += 1.;
                }
            }
            RebuildProgress::Completed => {
                self.rebuild_progress = None;
            }
        }
    }

    fn handle_crossterm_event(&mut self, event: event::Event) {
        match event {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(mouse_event) => {
                self.on_mouse_event(mouse_event);
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    fn on_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::ScrollDown => {
                self.move_up(3);
            }
            MouseEventKind::ScrollUp => {
                self.move_down(3);
            }
            _ => (),
        }
    }

    fn move_up(&mut self, interval: usize) {
        self.selected_item = self.selected_item.and_then(|n| n.checked_sub(interval));
    }

    fn move_down(&mut self, interval: usize) {
        let result = self
            .selected_item
            .and_then(|n| n.checked_add(interval))
            .map(|n| {
                if n < self.top_stories.len() {
                    n
                } else {
                    self.top_stories.len() - 1
                }
            });
        self.selected_item = result.or(Some(0));
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Down | KeyCode::Char('j')) => {
                self.move_down(1);
            }
            (_, KeyCode::Up | KeyCode::Char('k')) => {
                self.move_up(1);
            }
            (_, KeyCode::PageDown) => {
                self.move_down(10);
            }
            (_, KeyCode::PageUp) => {
                self.move_up(10);
            }
            (_, KeyCode::Home) => {
                self.selected_item = None;
            }
            (_, KeyCode::End) | (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                self.selected_item = Some(self.top_stories.len() - 1);
            }
            // Rebuild the index.
            (_, KeyCode::Char('r')) => {
                if self.rebuild_progress.is_none() {
                    self.event_manager
                        .rebuild_index(self.search_context.clone());
                }
            }
            (_, KeyCode::Char('o')) => {
                if let Some(url) = self.select_item_url()
                    && let Err(err) = open::that(url)
                {
                    eprintln!("Failed to open url {url}: {err}");
                }
            }

            _ => {}
        }
    }

    pub fn select_item_url(&self) -> Option<&str> {
        self.selected_item
            .and_then(|selected| self.top_stories.get(selected))
            .and_then(|item| item.url.as_deref())
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let main_layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(if self.rebuild_progress.is_some() {
                3
            } else {
                1
            }),
        ])
        .split(area);

        ArticlesWidget::new(self.selected_item).render(main_layout[0], buf, &mut self.top_stories);
        FooterWidget::new(self).render(main_layout[1], buf);
    }
}
