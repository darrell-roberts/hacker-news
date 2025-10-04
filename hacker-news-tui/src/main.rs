use std::sync::{Arc, RwLock};

use crate::articles::ArticlesWidget;
use app_dirs2::{AppInfo, get_app_dir};
use color_eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};
use hacker_news_api::ArticleType;
use hacker_news_search::{SearchContext, api::Story, api_client};
use ratatui::{
    DefaultTerminal,
    widgets::{StatefulWidget, Widget},
};

mod articles;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new()?.run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,
    search_context: Arc<RwLock<SearchContext>>,
    top_stories: Vec<Story>,
    // scroll_offset: usize,
    selected_item: Option<usize>,
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
            running: false,
            search_context,
            top_stories,
            // scroll_offset: 0,
            selected_item: None,
        })
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(mouse_event) => {
                self.on_mouse_event(mouse_event);
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::ScrollDown => {
                self.move_down(3);
            }
            MouseEventKind::ScrollUp => {
                self.move_up(3);
            }
            _ => (),
        }
    }

    fn move_up(&mut self, interval: usize) {
        self.selected_item = self.selected_item.and_then(|n| n.checked_sub(interval));
    }

    fn move_down(&mut self, interval: usize) {
        let result = self.selected_item.and_then(|n| n.checked_add(interval));
        self.selected_item = result.or(Some(0));
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Down) => {
                self.move_down(1);
            }
            (_, KeyCode::Up) => {
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
            (_, KeyCode::End) => {
                self.selected_item = Some(self.top_stories.len());
            }

            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        ArticlesWidget::new(
            self.search_context.clone(),
            // self.scroll_offset,
            self.selected_item,
        )
        .render(area, buf, &mut self.top_stories);
    }
}
