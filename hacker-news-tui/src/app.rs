//! App state, management and root widget.
use std::sync::{Arc, RwLock};

use crate::{
    articles::{ArticlesState, ArticlesWidget},
    comments::{CommentState, CommentsWidget},
    events::{AppEvent, EventManager, IndexRebuildState},
    footer::FooterWidget,
};
use app_dirs2::{AppInfo, get_app_dir};
use color_eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};
use hacker_news_api::ArticleType;
use hacker_news_search::{IndexStats, RebuildProgress, SearchContext, api_client};
use log::{debug, error};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{ScrollbarState, StatefulWidget, Widget},
};

/// The main application which holds the state and logic of the application.
pub struct App {
    event_manager: EventManager,
    /// Is the application running?
    running: bool,
    search_context: Arc<RwLock<SearchContext>>,
    pub rebuild_progress: Option<IndexRebuildState>,
    pub comment_state: Option<CommentState>,
    pub index_stats: Option<IndexStats>,
    articles_state: ArticlesState,
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

        let stories = search_context.read().unwrap().top_stories(75, 0)?;

        let articles_state = ArticlesState {
            stories,
            list_state: Default::default(),
            scrollbar_state: ScrollbarState::new(75),
        };

        Ok(Self {
            event_manager: EventManager::new(),
            running: false,
            search_context,
            rebuild_progress: None,
            comment_state: None,
            index_stats: None,
            articles_state,
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
            AppEvent::IndexingCompleted(index_stats) => {
                let top_stories = self.search_context.read().unwrap().top_stories(75, 0);
                match top_stories {
                    Ok(stories) => {
                        self.articles_state.stories = stories;
                    }
                    Err(err) => {
                        error!("Failed to fetch top stories: {err}");
                    }
                }
                self.index_stats.replace(index_stats);
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
                self.move_up(1);
            }
            MouseEventKind::ScrollUp => {
                self.move_down(1);
            }
            _ => (),
        }
    }

    fn move_up(&mut self, interval: usize) {
        match self.comment_state.as_mut() {
            Some(state) => {
                let mut position = state.scroll_view_state.offset();
                position.y = position.y.saturating_sub(interval as u16);
                state.scroll_view_state.set_offset(position);
            }
            None => {
                let selected = self
                    .articles_state
                    .list_state
                    .selected()
                    .map(|n| n.saturating_sub(interval));
                self.articles_state.list_state.select(selected);
                for _ in 0..interval {
                    self.articles_state.scrollbar_state.prev();
                }
            }
        }
    }

    fn move_down(&mut self, interval: usize) {
        match self.comment_state.as_mut() {
            Some(state) => {
                let mut position = state.scroll_view_state.offset();
                position.y = position.y.saturating_add(interval as u16);
                state.scroll_view_state.set_offset(position);
            }
            None => {
                let selected = self
                    .articles_state
                    .list_state
                    .selected()
                    .map(|n| n.saturating_add(interval));
                self.articles_state.list_state.select(selected.or(Some(0)));
                for _ in 0..interval {
                    self.articles_state.scrollbar_state.next();
                }
            }
        }
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                match self.comment_state.as_mut() {
                    Some(state) => {
                        let parent_id = state.child_stack.pop();
                        match parent_id {
                            Some(parent_id) => {
                                state.parent_id = parent_id;
                                let comments = self
                                    .search_context
                                    .read()
                                    .unwrap()
                                    .comments(parent_id, 10, 0);
                                match comments {
                                    Ok((comments, total)) => {
                                        state.comments = comments;
                                        state.total_comments = total;
                                    }
                                    Err(err) => {
                                        error!("Failed to get comments: {err}");
                                    }
                                }
                            }
                            None => {
                                self.comment_state = None;
                            }
                        }
                    }
                    None => {
                        self.quit();
                    }
                }
            }
            (_, KeyCode::Down | KeyCode::Char('j')) => {
                self.move_down(1);
            }
            (_, KeyCode::Up | KeyCode::Char('k')) => {
                self.move_up(1);
            }
            (_, KeyCode::PageDown) | (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
                self.move_down(10);
            }
            (_, KeyCode::PageUp)
            | (KeyModifiers::CONTROL, KeyCode::Char('b') | KeyCode::Char('u')) => {
                self.move_up(10);
            }
            (_, KeyCode::Home) => match self.comment_state.as_mut() {
                Some(state) => {
                    state.scroll_view_state.scroll_to_top();
                }
                None => {
                    self.articles_state.list_state.select(Some(0));
                    self.articles_state.scrollbar_state.first();
                }
            },
            (_, KeyCode::End) | (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                match self.comment_state.as_mut() {
                    Some(state) => {
                        state.scroll_view_state.scroll_to_bottom();
                    }
                    None => {
                        self.articles_state
                            .list_state
                            .select(Some(self.articles_state.stories.len() - 1));
                        self.articles_state.scrollbar_state.last();
                    }
                }
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
                    error!("Failed to open url {url}: {err}");
                }
            }
            (_, KeyCode::Right | KeyCode::Char('c')) => {
                match self.comment_state.as_mut() {
                    // The viewing comment is being requested to open children.
                    Some(state) => {
                        debug!("opening child comment");
                        if let Some(parent_id) = state
                            .viewing
                            .and_then(|viewing| state.comments.get(viewing as usize))
                            .map(|comment| comment.id)
                        {
                            state.child_stack.push(state.parent_id);
                            state.parent_id = parent_id;
                            state.offset = 0;
                            state.viewing = None;
                            debug!("opening comments for {parent_id}");
                            let comments = self.search_context.read().unwrap().comments(
                                state.parent_id,
                                state.limit,
                                state.offset,
                            );
                            match comments {
                                Ok((comments, total)) => {
                                    state.comments = comments;
                                    state.total_comments = total;
                                    debug!("opened {} child comments", state.comments.len());
                                }
                                Err(err) => {
                                    error!("Failed to get comments: {err}");
                                }
                            }
                        }
                    }
                    // We are opening comments for a story
                    None => {
                        if let Some(selected_item) = self
                            .articles_state
                            .list_state
                            .selected()
                            .and_then(|id| self.articles_state.stories.get(id))
                            .map(|story| story.id)
                        {
                            debug!("opening comments for selected article: {selected_item}");
                            let comments =
                                self.search_context
                                    .read()
                                    .unwrap()
                                    .comments(selected_item, 10, 0);

                            match comments {
                                Ok((comments, total)) => {
                                    self.comment_state = Some(CommentState {
                                        parent_id: selected_item,
                                        limit: 10,
                                        offset: 0,
                                        viewing: None,
                                        comments,
                                        total_comments: total,
                                        scroll_view_state: Default::default(),
                                        child_stack: Default::default(),
                                    });
                                }
                                Err(err) => {
                                    error!("Failed to get comments: {err}");
                                }
                            }
                        }
                    }
                }
            }
            (_, KeyCode::Left) => {
                debug!("Closing comment view");
                self.comment_state = None;
            }
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                if let Some(state) = self.comment_state.as_mut()
                    && let Some(n) = state.viewing.as_mut()
                {
                    *n = n.saturating_sub(1);
                }
            }
            (_, KeyCode::Tab) => {
                if let Some(state) = self.comment_state.as_mut() {
                    match state.viewing.as_mut() {
                        Some(n) => {
                            *n = n.saturating_add(1);
                        }
                        None => {
                            state.viewing.replace(0);
                        }
                    }
                }
            }

            _ => {}
        }
    }

    pub fn select_item_url(&self) -> Option<&str> {
        self.articles_state
            .list_state
            .selected()
            .and_then(|selected| self.articles_state.stories.get(selected))
            .and_then(|item| item.url.as_deref())
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [content_area, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(if self.rebuild_progress.is_some() {
                3
            } else {
                1
            }),
        ])
        .areas(area);

        match self.comment_state.as_mut() {
            Some(comment_state) => {
                let selected_title = self
                    .articles_state
                    .list_state
                    .selected()
                    .and_then(|id| self.articles_state.stories.get(id))
                    .map(|story| story.title.as_str());
                CommentsWidget::new(selected_title.unwrap_or_default()).render(
                    content_area,
                    buf,
                    comment_state,
                );
            }
            None => {
                ArticlesWidget.render(content_area, buf, &mut self.articles_state);
            }
        }
        FooterWidget::new(self).render(footer_area, buf);
    }
}
