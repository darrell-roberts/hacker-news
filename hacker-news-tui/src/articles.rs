//! Articles list widget.
use hacker_news_search::api::Story;
use ratatui::{
    style::{Style, Stylize as _},
    text::{Line, Span},
    widgets::{
        Block, List, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
    },
};

/// Widget to render list of articles.
pub struct ArticlesWidget {
    list_state: ListState,
}

impl ArticlesWidget {
    /// Create a new articles widget with optional selected item.
    pub fn new(selected: Option<usize>) -> Self {
        Self {
            list_state: ListState::default().with_selected(selected),
        }
    }
}

impl StatefulWidget for &mut ArticlesWidget {
    type State = Vec<Story>;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let items = state
            .iter()
            .map(|item| render_article_line(item))
            .collect::<Vec<_>>();

        let title = Line::from("Hacker News").bold().blue().centered();

        List::new(items)
            .block(Block::bordered().title(title))
            .highlight_style(Style::new().yellow())
            .render(area, buf, &mut self.list_state);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state =
            ScrollbarState::new(state.len()).position(self.list_state.offset());

        scrollbar.render(area, buf, &mut scrollbar_state);
    }
}

/// Render a single line for an article.
fn render_article_line(article: &Story) -> Line<'_> {
    let style = Style::new().white();
    Line::from_iter([
        comment_col(article.descendants, style),
        Span::styled(&article.title, style),
        Span::styled(format!(" by {} ", &article.by), style.italic()),
        Span::styled(article.age_label().unwrap_or_default(), style.italic()),
    ])
}

/// Render the article total comment count column.
fn comment_col<'a>(comments: u64, style: Style) -> Span<'a> {
    if comments > 0 {
        Span::styled(format!("[{:<5}] ", comments), style)
    } else {
        Span::styled("        ", style)
    }
}
