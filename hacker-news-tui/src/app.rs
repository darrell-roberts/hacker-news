//! App state, management and root widget.
use crate::{
    articles::{ArticlesState, ArticlesWidget},
    comments::{CommentStack, CommentState, CommentsWidget},
    config::CONFIG_FILE,
    events::{AppEvent, EventManager, IndexRebuildState},
    footer::FooterWidget,
    search::{InputMode, SearchState, SearchWidget},
};
use color_eyre::Result;
use hacker_news_config::{save_config, search_context};
use hacker_news_search::{IndexStats, RebuildProgress, SearchContext, api_client};
use log::{debug, error};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{ListState, ScrollbarState, StatefulWidget, Widget},
};
use ratatui::{
    crossterm::event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
    },
    layout::Position,
};
use std::{
    ops::Not as _,
    sync::{Arc, RwLock},
};
use tui_input::backend::crossterm::EventHandler;

/// Active viewing state.
pub enum Viewing {
    /// Viewing comments.
    Comments(CommentState),
    /// Viewing search.
    Search(SearchState),
}

/// The main application which holds the state and logic of the application.
pub struct App {
    event_manager: EventManager,
    /// Is the application running?
    running: bool,
    search_context: Arc<RwLock<SearchContext>>,
    pub rebuild_progress: Option<IndexRebuildState>,
    // pub comment_state: Option<CommentState>,
    pub viewing_state: Option<Viewing>,
    pub index_stats: Option<IndexStats>,
    articles_state: ArticlesState,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(config: Option<IndexStats>) -> Result<Self, Box<dyn std::error::Error>> {
        let _ = api_client();

        let search_context = search_context()?;
        let stories = search_context.read().unwrap().top_stories(75, 0)?;

        let articles_state = ArticlesState {
            list_state: ListState::default().with_selected(stories.is_empty().not().then_some(0)),
            stories,
            scrollbar_state: ScrollbarState::new(75),
            page_height: 0,
        };

        Ok(Self {
            event_manager: EventManager::new(),
            running: false,
            search_context,
            rebuild_progress: None,
            viewing_state: None,
            index_stats: config,
            articles_state,
        })
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| {
                if let Some(Viewing::Search(state)) = self.viewing_state.as_ref()
                    && let InputMode::Editing = state.input_mode
                {
                    let cursor = state.input.visual_cursor();
                    frame.set_cursor_position(Position {
                        x: cursor as u16 + 1,
                        y: 1,
                    });
                }

                frame.render_widget(&mut self, frame.area());
            })?;
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
                        if !stories.is_empty() {
                            self.articles_state.list_state.select(Some(0));
                        }
                        self.articles_state.stories = stories;
                    }
                    Err(err) => {
                        error!("Failed to fetch top stories: {err}");
                    }
                }
                tokio::spawn(async move {
                    if let Err(err) = save_config(index_stats, CONFIG_FILE).await {
                        error!("Failed to save config: {err}");
                    }
                });

                self.index_stats.replace(index_stats);
            }
            AppEvent::StoryUpdated(story) => {
                if let Some(s) = self
                    .articles_state
                    .stories
                    .iter_mut()
                    .find(|s| s.id == story.id)
                {
                    *s = story;
                }
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
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                match self.viewing_state.as_mut() {
                    Some(Viewing::Search(search_state))
                        if matches!(search_state.input_mode, InputMode::Editing) =>
                    {
                        match key.code {
                            KeyCode::Esc => {
                                search_state.search = None;
                                search_state.input_mode = InputMode::Normal;
                            }
                            KeyCode::Enter => {
                                let search = search_state.input.value_and_reset();
                                match self.search_context.read().unwrap().search_all_comments(
                                    &search,
                                    10,
                                    search_state.offset,
                                ) {
                                    Ok((comments, total_comments)) => {
                                        search_state.comments = comments;
                                        search_state.total_comments = total_comments;
                                    }
                                    Err(err) => {
                                        error!("Failed to search: {err}");
                                    }
                                }
                                search_state.search = Some(search);
                                search_state.input_mode = InputMode::Normal;
                            }
                            _ => {
                                search_state.input.handle_event(&event);
                            }
                        }
                    }
                    _ => {
                        self.on_key_event(key);
                    }
                }
            }
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

    fn move_up(&mut self, interval: u16) {
        match self.viewing_state.as_mut() {
            Some(Viewing::Comments(state)) => {
                let mut position = state.scroll_view_state.offset();
                position.y = position.y.saturating_sub(interval);
                state.scroll_view_state.set_offset(position);
            }
            Some(Viewing::Search(state)) => {
                let mut position = state.scroll_view_state.offset();
                position.y = position.y.saturating_sub(interval);
                state.scroll_view_state.set_offset(position);
            }
            None => {
                let selected = self
                    .articles_state
                    .list_state
                    .selected()
                    .map(|n| n.saturating_sub(interval as usize));
                self.articles_state.list_state.select(selected);
                for _ in 0..interval {
                    self.articles_state.scrollbar_state.prev();
                }
            }
        }
    }

    fn move_down(&mut self, interval: u16) {
        match self.viewing_state.as_mut() {
            Some(Viewing::Comments(state)) => {
                let mut position = state.scroll_view_state.offset();
                position.y = position.y.saturating_add(interval);
                state.scroll_view_state.set_offset(position);
            }
            Some(Viewing::Search(state)) => {
                let mut position = state.scroll_view_state.offset();
                position.y = position.y.saturating_add(interval);
                state.scroll_view_state.set_offset(position);
            }
            None => {
                let selected = self
                    .articles_state
                    .list_state
                    .selected()
                    .and_then(|n| u16::try_from(n).ok())
                    .map(|n| n.saturating_add(interval))
                    .or((interval > 1).then_some(interval));
                self.articles_state
                    .list_state
                    .select(selected.or(Some(0_u16)).map(|n| n as usize));
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
                match self.viewing_state.as_mut() {
                    Some(Viewing::Comments(state)) => {
                        let stack_parent = state.child_stack.pop();
                        match stack_parent {
                            Some(CommentStack {
                                parent_id,
                                offset,
                                scroll_view_state,
                            }) => {
                                let last_parent_id = state.parent_id;
                                state.parent_id = parent_id;
                                state.offset = offset;
                                state.scroll_view_state = scroll_view_state;

                                let mut next_offset = offset;

                                loop {
                                    debug!("Trying offset: {next_offset}");
                                    let (comments, total_comments) = self
                                        .search_context
                                        .read()
                                        .unwrap()
                                        .comments(parent_id, 10, next_offset)
                                        .unwrap();

                                    if let Some(selected_index) = comments
                                        .iter()
                                        .position(|comment| comment.id == last_parent_id)
                                    {
                                        debug!("Updating viewing index");
                                        state.viewing = Some(selected_index as u64);
                                        state.comments = comments;
                                        state.total_comments = total_comments;
                                        break;
                                    }
                                    next_offset += 10;
                                }
                            }
                            None => {
                                self.viewing_state = None;
                            }
                        }
                    }
                    Some(Viewing::Search(_state)) => {
                        self.viewing_state = None;
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
                match self.viewing_state.as_ref() {
                    Some(Viewing::Comments(state)) => {
                        self.move_down(state.page_height);
                    }
                    Some(Viewing::Search(state)) => {
                        self.move_down(state.page_height);
                    }
                    None => {
                        self.move_down(self.articles_state.page_height);
                    }
                };
            }
            (_, KeyCode::PageUp)
            | (KeyModifiers::CONTROL, KeyCode::Char('b') | KeyCode::Char('u')) => {
                match self.viewing_state.as_ref() {
                    Some(Viewing::Comments(state)) => {
                        self.move_up(state.page_height);
                    }
                    Some(Viewing::Search(state)) => {
                        self.move_up(state.page_height);
                    }
                    None => {
                        self.move_up(self.articles_state.page_height);
                    }
                }
            }
            (_, KeyCode::Home) => match self.viewing_state.as_mut() {
                Some(Viewing::Comments(state)) => {
                    state.scroll_view_state.scroll_to_top();
                }
                Some(Viewing::Search(state)) => {
                    state.scroll_view_state.scroll_to_top();
                }
                None => {
                    self.articles_state.list_state.select(Some(0));
                    self.articles_state.scrollbar_state.first();
                }
            },
            (_, KeyCode::End) | (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                match self.viewing_state.as_mut() {
                    Some(Viewing::Comments(state)) => {
                        state.scroll_view_state.scroll_to_bottom();
                    }
                    Some(Viewing::Search(state)) => {
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
            (_, KeyCode::Char('c')) => {
                match self.viewing_state.as_mut() {
                    // The viewing comment is being requested to open children.
                    Some(Viewing::Comments(state)) => {
                        if let Some(parent_id) = state
                            .viewing
                            .and_then(|viewing| state.comments.get(viewing as usize))
                            .filter(|comment| !comment.kids.is_empty())
                            .map(|comment| comment.id)
                        {
                            state.child_stack.push(CommentStack {
                                parent_id: state.parent_id,
                                offset: state.offset,
                                scroll_view_state: state.scroll_view_state,
                            });
                            state.parent_id = parent_id;
                            state.offset = 0;
                            state.viewing = None;
                            let comments = self.search_context.read().unwrap().comments(
                                state.parent_id,
                                state.limit,
                                state.offset,
                            );
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
                    }
                    Some(Viewing::Search(_state)) => {}
                    // We are opening comments for a story
                    None => {
                        if let Some(selected_item) = self
                            .articles_state
                            .list_state
                            .selected()
                            .and_then(|id| self.articles_state.stories.get(id))
                            .map(|story| story.id)
                        {
                            let comments =
                                self.search_context
                                    .read()
                                    .unwrap()
                                    .comments(selected_item, 10, 0);

                            match comments {
                                Ok((comments, total)) => {
                                    self.viewing_state = Some(Viewing::Comments(CommentState {
                                        parent_id: selected_item,
                                        limit: 10,
                                        offset: 0,
                                        viewing: None,
                                        comments,
                                        total_comments: total,
                                        scroll_view_state: Default::default(),
                                        child_stack: Default::default(),
                                        page_height: 0,
                                    }));
                                }
                                Err(err) => {
                                    error!("Failed to get comments: {err}");
                                }
                            }
                        }
                    }
                }
            }
            (_, KeyCode::Right) => {
                if let Some(viewing) = self.viewing_state.as_mut() {
                    match viewing {
                        Viewing::Comments(comment_state) => {
                            comment_state.page_forward(self.search_context.clone());
                        }
                        Viewing::Search(search_state) => {
                            search_state.page_forward(self.search_context.clone());
                        }
                    }
                }
            }
            (_, KeyCode::Left) => {
                if let Some(viewing) = self.viewing_state.as_mut() {
                    match viewing {
                        Viewing::Comments(comment_state) => {
                            comment_state.page_back(self.search_context.clone());
                        }
                        Viewing::Search(search_state) => {
                            search_state.page_back(self.search_context.clone());
                        }
                    }
                }
            }
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                if let Some(viewing) = self.viewing_state.as_mut() {
                    match viewing {
                        Viewing::Comments(comment_state) => {
                            if let Some(n) = comment_state.viewing.as_mut() {
                                *n = n.saturating_sub(1);
                            }
                        }
                        Viewing::Search(search_state) => {
                            if let Some(n) = search_state.viewing.as_mut() {
                                *n = n.saturating_sub(1);
                            }
                        }
                    }
                }
            }
            (_, KeyCode::Tab) => {
                if let Some(viewing) = self.viewing_state.as_mut() {
                    match viewing {
                        Viewing::Comments(comment_state) => {
                            match comment_state.viewing.as_mut() {
                                Some(n) => {
                                    let next_val = n.saturating_add(1);
                                    if next_val < comment_state.comments.len() as u64 {
                                        *n = n.saturating_add(1);
                                    }
                                }
                                None => {
                                    comment_state.viewing.replace(0);
                                }
                            };
                        }
                        Viewing::Search(search_state) => {
                            match search_state.viewing.as_mut() {
                                Some(n) => {
                                    let next_val = n.saturating_add(1);
                                    if next_val < search_state.comments.len() as u64 {
                                        *n = n.saturating_add(1);
                                    }
                                }
                                None => {
                                    search_state.viewing.replace(0);
                                }
                            };
                        }
                    }
                }
            }
            // Update the selected story in the stories view
            (_, KeyCode::Char('u')) => {
                if self.viewing_state.is_none() {
                    let story = self
                        .articles_state
                        .list_state
                        .selected()
                        .and_then(|selected| self.articles_state.stories.get(selected))
                        .cloned();
                    if let Some(story) = story {
                        self.event_manager
                            .update_story(self.search_context.clone(), story);
                    }
                };
            }
            // Open search view
            (_, KeyCode::Char('/')) => {
                self.viewing_state = Some(Viewing::Search(SearchState::default()));
            }
            // Rebuild comment stack on search result comment
            (_, KeyCode::Char('t')) => {
                if let Some(viewing) = self.viewing_state.as_mut()
                    && let Viewing::Search(search_state) = viewing
                    && let Some((comment_id, comment_parent_id)) = search_state
                        .viewing
                        .and_then(|index| search_state.comments.get(index as usize))
                        .map(|comment| (comment.id, comment.parent_id))
                {
                    let result = self.search_context.read().unwrap().parents(comment_id);
                    match result {
                        Ok(stack) => {
                            let child_stack = || {
                                stack
                                    .comments
                                    .iter()
                                    .rev()
                                    .scan(stack.story.id, |parent_id, comment| {
                                        let current_parent_id = *parent_id;
                                        *parent_id = comment.parent_id;
                                        Some(CommentStack {
                                            parent_id: current_parent_id,
                                            ..Default::default()
                                        })
                                    })
                                    .skip(1)
                                    .collect::<Vec<_>>()
                            };

                            let mut next_offset = 0;

                            loop {
                                let (comments, total_comments) = self
                                    .search_context
                                    .read()
                                    .unwrap()
                                    .comments(comment_parent_id, 10, next_offset)
                                    .unwrap();
                                let viewing = comments
                                    .iter()
                                    .position(|comment| comment.id == comment_id)
                                    .map(|id| id as u64);
                                if viewing.is_some() {
                                    let comments_state = CommentState {
                                        parent_id: comment_parent_id,
                                        comments,
                                        total_comments,
                                        child_stack: child_stack(),
                                        limit: 10,
                                        viewing,
                                        ..Default::default()
                                    };
                                    let selected_index = self
                                        .articles_state
                                        .stories
                                        .iter()
                                        .position(|story| story.id == stack.story.id);
                                    self.articles_state.list_state.select(selected_index);
                                    self.viewing_state = Some(Viewing::Comments(comments_state));
                                    break;
                                } else {
                                    next_offset += 10;
                                }
                            }
                        }
                        Err(err) => {
                            error!("Failed to build comment thread stack: {err}");
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
                2
            }),
        ])
        .areas(area);

        match self.viewing_state.as_mut() {
            Some(Viewing::Comments(comment_state)) => {
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
            Some(Viewing::Search(state)) => {
                SearchWidget.render(content_area, buf, state);
            }
            None => {
                ArticlesWidget.render(content_area, buf, &mut self.articles_state);
            }
        }
        FooterWidget::new(self).render(footer_area, buf);
    }
}
