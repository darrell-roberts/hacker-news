use std::sync::{Arc, RwLock};

use hacker_news_search::{SearchContext, api::Story};
use ratatui::{
    style::{Style, Stylize as _},
    text::{Line, Span},
    widgets::{
        Block, List, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
    },
};

pub struct ArticlesWidget {
    pub search_context: Arc<RwLock<SearchContext>>,
    list_state: ListState,
}

impl ArticlesWidget {
    pub fn new(search_context: Arc<RwLock<SearchContext>>, offset: usize) -> Self {
        Self {
            search_context,
            list_state: ListState::default().with_offset(offset),
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
            .render(area, buf, &mut self.list_state);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state =
            ScrollbarState::new(state.len()).position(self.list_state.offset());

        scrollbar.render(area, buf, &mut scrollbar_state);
    }
}

fn render_article_line(article: &Story) -> Line<'_> {
    let style = Style::new().white();
    Line::from(
        [
            Span::styled(&article.title, style),
            Span::styled(format!(" by {}", &article.by), style.italic()),
        ]
        .into_iter()
        .collect::<Vec<_>>(),
    )
}
