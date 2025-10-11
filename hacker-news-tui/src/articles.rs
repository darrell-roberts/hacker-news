//! Articles list widget.
use hacker_news_search::api::{AgeLabel as _, Story};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
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
    pub page_height: u16,
}

/// Widget to render list of articles.
pub struct ArticlesWidget;

impl StatefulWidget for &mut ArticlesWidget {
    type State = ArticlesState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items = state
            .stories
            .iter()
            .zip(1..)
            .map(|(item, index)| render_article_line(item, index))
            .collect::<Vec<_>>();

        let title = Line::from("Hacker News").bold().blue().centered();

        let [content, scroll] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(2)]).areas(area);

        state.page_height = area.height - 2;

        List::new(items)
            .block(Block::bordered().title(title))
            .highlight_style(Style::new().green().on_black())
            .render(content, buf, &mut state.list_state);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        scrollbar.render(scroll, buf, &mut state.scrollbar_state);
    }
}

/// Render a single line for an article.
fn render_article_line(article: &Story, index: usize) -> Line<'_> {
    let style = Style::new().white();
    Line::from_iter([
        Span::raw(format!("{index:<3}")),
        Span::styled(&article.title, style),
        Span::styled(format!(" by {} ", &article.by), style.italic()),
        Span::styled(article.age_label().unwrap_or_default(), style.italic()),
        if article.descendants > 0 {
            Span::raw(format!(" [{}]", article.descendants))
        } else {
            Span::raw("")
        },
    ])
}
