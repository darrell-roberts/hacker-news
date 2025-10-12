//! Comment search view and state
use hacker_news_search::api::Comment;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{StatefulWidget, Widget},
};
use tui_scrollview::ScrollViewState;

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
}

pub struct SearchWidget;

impl StatefulWidget for SearchWidget {
    type State = SearchState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        Line::raw("Search").render(area, buf);
    }
}
