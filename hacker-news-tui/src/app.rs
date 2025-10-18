//! App state, management and root widget.
use crate::{
    articles::{ArticlesState, ArticlesWidget},
    comments::{CommentStack, CommentState, CommentsWidget},
    config::{Config, save_config},
    events::{AppEvent, EventManager, IndexRebuildState},
    footer::FooterWidget,
    help::HelpWidget,
    search::{InputMode, SearchState, SearchWidget},
};
use color_eyre::Result;
use hacker_news_config::search_context;
use hacker_news_search::{RebuildProgress, SearchContext, api_client};
use log::error;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Clear, ListState, ScrollbarState, StatefulWidget, Widget},
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

/// Active view
#[derive(Clone, Copy)]
pub enum View {
    Articles,
    Search,
    Comments,
}

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
    pub search_context: Arc<RwLock<SearchContext>>,
    pub rebuild_progress: Option<IndexRebuildState>,
    pub viewing_state: Option<Viewing>,
    pub config: Config,
    articles_state: ArticlesState,
    show_help: bool,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let _ = api_client();

        let search_context = search_context()?;
        let stories = search_context.read().unwrap().top_stories(75, 0)?;

        let articles_state = ArticlesState {
            list_state: ListState::default().with_selected(stories.is_empty().not().then_some(0)),
            stories,
            scrollbar_state: ScrollbarState::new(75),
            page_height: 0,
            article_type: hacker_news_api::ArticleType::Top,
        };

        Ok(Self {
            event_manager: EventManager::new(),
            running: false,
            search_context,
            rebuild_progress: None,
            viewing_state: None,
            config,
            articles_state,
            show_help: false,
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

                if self.show_help {
                    // show help
                    let area = centered_rect(50, 50, frame.area());
                    frame.render_widget(Clear, area);
                    frame.render_widget(HelpWidget::new(self.viewing()), area);
                }
            })?;
            self.handle_event(self.event_manager.next()?);
        }
        Ok(())
    }

    fn viewing(&self) -> View {
        match &self.viewing_state {
            Some(state) => match state {
                Viewing::Comments(_) => View::Comments,
                Viewing::Search(_) => View::Search,
            },
            None => View::Articles,
        }
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

                let existing_stat = self
                    .config
                    .index_config
                    .index_stats
                    .iter_mut()
                    .find(|stat| stat.category == index_stats.category);
                match existing_stat {
                    Some(stat) => {
                        *stat = index_stats;
                    }
                    None => {
                        self.config.index_config.index_stats.push(index_stats);
                    }
                }

                let config = self.config.clone();

                tokio::spawn(async {
                    if let Err(err) = save_config(config).await {
                        error!("Failed to save config: {err}");
                    }
                });
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
        // Quit app or close child comment or search result.
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                if self.show_help {
                    self.show_help = false;
                    return;
                }

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
                                state.scroll_view_state = scroll_view_state;

                                let mut current_offset = offset;

                                loop {
                                    let (comments, total_comments) = self
                                        .search_context
                                        .read()
                                        .unwrap()
                                        .comments(parent_id, 10, current_offset)
                                        .unwrap();

                                    let selected_index = comments
                                        .iter()
                                        .position(|comment| comment.id == last_parent_id);

                                    if selected_index.is_some() {
                                        state.viewing = selected_index;
                                        state.comments = comments;
                                        state.total_comments = total_comments;
                                        state.offset = current_offset;
                                        break;
                                    }
                                    current_offset += 10;
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
            // Scroll down.
            (_, KeyCode::Down | KeyCode::Char('j')) => {
                self.move_down(1);
            }
            // Scroll up.
            (_, KeyCode::Up | KeyCode::Char('k')) => {
                self.move_up(1);
            }
            // Page down in scroll.
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
            // Page up in scroll.
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
            // Select first item.
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
            // Select last item.
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
                    self.event_manager.rebuild_index(
                        self.search_context.clone(),
                        self.articles_state.article_type,
                    );
                }
            }
            // Open URL for story.
            (_, KeyCode::Char('o')) => {
                if let Some(url) = self.select_item_url()
                    && let Err(err) = open::that(url)
                {
                    error!("Failed to open url {url}: {err}");
                }
            }
            // Open child comments.
            (_, KeyCode::Char('c')) => {
                match self.viewing_state.as_mut() {
                    // The viewing comment is being requested to open children.
                    Some(Viewing::Comments(state)) => {
                        if let Some(parent_id) = state
                            .viewing
                            .and_then(|viewing| state.comments.get(viewing))
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
                                        comments,
                                        total_comments: total,
                                        ..Default::default()
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
            // Previous page in paginated results.
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
                } else {
                    self.articles_state.next_article_type();
                    self.update_stories();
                }
            }
            // Next page in paginated results.
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
                } else {
                    self.articles_state.previous_article_type();
                    self.update_stories();
                }
            }
            // Move selection up.
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
            // Move selection down.
            (_, KeyCode::Tab) => {
                if let Some(viewing) = self.viewing_state.as_mut() {
                    match viewing {
                        Viewing::Comments(comment_state) => {
                            match comment_state.viewing.as_mut() {
                                Some(n) => {
                                    let next_val = n.saturating_add(1);
                                    if next_val < comment_state.comments.len() {
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
                                    if next_val < search_state.comments.len() {
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
                        .and_then(|index| search_state.comments.get(index))
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

                            let mut current_offset = 0;

                            loop {
                                let (comments, total_comments) = self
                                    .search_context
                                    .read()
                                    .unwrap()
                                    .comments(comment_parent_id, 10, current_offset)
                                    .unwrap();
                                let viewing =
                                    comments.iter().position(|comment| comment.id == comment_id);
                                if viewing.is_some() {
                                    let comments_state = CommentState {
                                        parent_id: comment_parent_id,
                                        comments,
                                        total_comments,
                                        child_stack: child_stack(),
                                        limit: 10,
                                        viewing,
                                        offset: current_offset,
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
                                    current_offset += 10;
                                }
                            }
                        }
                        Err(err) => {
                            error!("Failed to build comment thread stack: {err}");
                        }
                    }
                }
            }
            (_, KeyCode::Char('?')) => {
                if let Some(Viewing::Search(state)) = &self.viewing_state
                    && matches!(state.input_mode, InputMode::Editing)
                {
                    // ignore showing help if we are editing a search input.
                } else {
                    self.show_help = true;
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

    fn update_stories(&mut self) {
        self.search_context
            .write()
            .unwrap()
            .activate_index(self.articles_state.article_type)
            .unwrap();
        match self.search_context.read().unwrap().top_stories(75, 0) {
            Ok(stories) => {
                self.articles_state.stories = stories;
                self.articles_state.list_state.select(Some(0));
            }
            Err(err) => {
                error!("Failed to fetch stories: {err}");
            }
        }
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
                let (selected_title, selected_body) = self
                    .articles_state
                    .list_state
                    .selected()
                    .and_then(|id| self.articles_state.stories.get(id))
                    .map(|story| (story.title.as_str(), story.body.as_deref()))
                    .unwrap_or_default();

                CommentsWidget::new(selected_title, selected_body).render(
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Length(18),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Length(45),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
