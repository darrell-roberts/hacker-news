//! Comment search view and state
use std::sync::{Arc, RwLock};

use hacker_news_search::{SearchContext, api::Comment};
use log::error;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect, Size},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{StatefulWidget, Widget},
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
    pub viewing: Option<u64>,
    pub comments: Vec<Comment>,
    pub total_comments: usize,
    pub scroll_view_state: ScrollViewState,
    pub page_height: u16,
    pub input: Input,
    pub input_mode: InputMode,
}

impl SearchState {
    pub fn page_forward(&mut self, search_context: Arc<RwLock<SearchContext>>) {
        self.update_offset(self.offset.saturating_add(10));
        self.update_comments(search_context);
    }

    pub fn page_back(&mut self, search_context: Arc<RwLock<SearchContext>>) {
        self.update_offset(self.offset.saturating_sub(10));
        self.update_comments(search_context);
    }

    fn update_offset(&mut self, next_offset: usize) {
        let total_pages = self.total_comments / 10;
        if next_offset / 10 < total_pages {
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
}

pub struct SearchWidget;

impl StatefulWidget for SearchWidget {
    type State = SearchState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [search_area, page_area, search_results] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        match state.input_mode {
            InputMode::Editing => {
                let val = state.input.value();
                Line::raw(val).render(search_area, buf);
                let _x = state.input.visual_cursor();
            }
            InputMode::Normal => {
                Line::raw(state.search.as_deref().unwrap_or_default()).render(search_area, buf);
                Line::raw(format!("Search results {}", state.comments.len()))
                    .render(search_results, buf);
            }
        }

        if state.total_comments > 0 {
            let total_pages = state.total_comments / 10;
            let selected_page = if state.offset == 0 {
                1
            } else {
                state.offset / 10 + 1
            };
            let spans = (1..=total_pages).map(|page| {
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
        render_comments(buf, state, search_results);
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
