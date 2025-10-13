//! Comment search view and state
use std::sync::{Arc, RwLock};

use hacker_news_search::{SearchContext, api::Comment};
use log::error;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect, Size},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, StatefulWidget, Widget, block::Title},
};
use tui_input::Input;
use tui_scrollview::ScrollViewState;

use crate::comments::render_comment;

#[derive(Default)]
pub enum InputMode {
    #[default]
    Editing,
    Normal,
}

/// Comment search state.
#[derive(Default)]
pub struct SearchState {
    pub search: Option<String>,
    pub limit: usize,
    pub offset: usize,
    pub viewing: Option<usize>,
    pub comments: Vec<Comment>,
    pub total_comments: usize,
    pub scroll_view_state: ScrollViewState,
    pub page_height: u16,
    pub input: Input,
    pub input_mode: InputMode,
}

impl SearchState {
    pub fn page_forward(&mut self, search_context: Arc<RwLock<SearchContext>>) {
        self.viewing = None;
        self.update_offset(self.offset.saturating_add(10));
        self.update_comments(search_context);
        self.scroll_view_state.scroll_to_top();
    }

    pub fn page_back(&mut self, search_context: Arc<RwLock<SearchContext>>) {
        self.viewing = None;
        self.update_offset(self.offset.saturating_sub(10));
        self.update_comments(search_context);
    }

    fn update_offset(&mut self, next_offset: usize) {
        if next_offset / 10 < self.total_pages() {
            self.offset = next_offset;
        }
    }

    fn update_comments(&mut self, search_context: Arc<RwLock<SearchContext>>) {
        let result = search_context.read().unwrap().search_all_comments(
            self.search.as_deref().unwrap_or_default(),
            10,
            self.offset,
        );
        match result {
            Ok((comments, total_comments)) => {
                self.comments = comments;
                self.total_comments = total_comments;
            }
            Err(err) => {
                error!("Failed to get comments: {err}");
            }
        }
    }

    fn total_pages(&self) -> usize {
        let remainder = self.total_comments % 10;
        self.total_comments / 10 + if remainder > 0 { 1 } else { 0 }
    }

    fn selected_page(&self) -> usize {
        if self.offset == 0 {
            1
        } else {
            self.offset / 10 + 1
        }
    }
}

/// Search Widget
pub struct SearchWidget;

impl StatefulWidget for SearchWidget {
    type State = SearchState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [search_area, search_results, page_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        // Search input/display.
        match state.input_mode {
            InputMode::Editing => {
                let val = state.input.value();
                Paragraph::new(val)
                    .block(Block::bordered().title(Title::from("Search")))
                    .render(search_area, buf);
            }
            InputMode::Normal => {
                Paragraph::new(state.search.as_deref().unwrap_or_default())
                    .block(Block::bordered().title(Title::from("Search")))
                    .render(search_area, buf);
            }
        }

        // Search comments results.
        render_comments(buf, state, search_results);

        // Pagination pages.
        if state.total_comments > 0 {
            let selected_page = state.selected_page();
            let spans = (1..=state.total_pages()).map(|page| {
                Span::styled(
                    format!("{page} "),
                    if page == selected_page {
                        Style::default().bold().magenta()
                    } else {
                        Style::default()
                    },
                )
            });

            Line::from_iter(spans).centered().render(page_area, buf);
        }
    }
}

fn render_comments(buf: &mut Buffer, state: &mut SearchState, body: Rect) {
    let paragraph_widgets = state
        .comments
        .iter()
        .zip(0..)
        .map(|(item, index)| render_comment(item, state.viewing == Some(index)))
        .collect::<Vec<_>>();

    let scroll_view_height: u16 = paragraph_widgets
        .iter()
        .map(|p| p.line_count(buf.area.width))
        .sum::<usize>() as u16
        + 5;

    let width = if buf.area.height < scroll_view_height {
        buf.area.width - 1
    } else {
        buf.area.width
    };

    let mut scroll_view = tui_scrollview::ScrollView::new(Size::new(width, scroll_view_height));
    let mut y = 0;
    let paragraph_width = width - 2;

    for paragraph in paragraph_widgets {
        let height = paragraph.line_count(paragraph_width);
        scroll_view.render_widget(
            paragraph,
            Rect {
                x: 0,
                y,
                width: paragraph_width,
                height: height as u16,
            },
        );
        y += height as u16;
    }

    state.page_height = scroll_view.area().height;
    scroll_view.render(body, buf, &mut state.scroll_view_state);
}
