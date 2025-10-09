//! Articles list widget.
use hacker_news_search::api::{AgeLabel as _, Story};
use ratatui::{
    style::{Style, Stylize as _},
    text::{Line, Span},
    widgets::{
        Block, List, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
    },
};

#[derive(Default)]
pub struct ArticlesState {
    pub stories: Vec<Story>,
    pub list_state: ListState,
    pub scrollbar_state: ScrollbarState,
}

/// Widget to render list of articles.
pub struct ArticlesWidget;

impl StatefulWidget for &mut ArticlesWidget {
    type State = ArticlesState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let items = state
            .stories
            .iter()
            .map(|item| render_article_line(item))
            .collect::<Vec<_>>();

        let title = Line::from("Hacker News").bold().blue().centered();

        List::new(items)
            .block(Block::bordered().title(title))
            .highlight_style(Style::new().green().on_black())
            .render(area, buf, &mut state.list_state);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        scrollbar.render(area, buf, &mut state.scrollbar_state);
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
