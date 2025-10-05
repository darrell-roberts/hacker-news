//! Comments view widget.
use hacker_news_search::api::Comment;
use html_sanitizer::Element;
use log::debug;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, StatefulWidget, Wrap},
};
use tui_scrollview::{ScrollViewState, ScrollbarVisibility};

pub struct CommentState {
    pub parent_id: u64,
    pub limit: usize,
    pub offset: usize,
    pub viewing: Option<u64>,
    pub comments: Vec<Comment>,
    pub total_comments: usize,
    pub scroll_view_state: ScrollViewState,
    // pub list_state: ListState,
}

#[derive(Default)]
pub struct CommentsWidget {}

impl StatefulWidget for &mut CommentsWidget {
    type State = CommentState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let mut scroll_view = tui_scrollview::ScrollView::new(area.as_size())
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never)
            .vertical_scrollbar_visibility(ScrollbarVisibility::Always);
        let mut y = 0;

        let comment_height = 10;

        for paragraph in state.comments.iter().map(|item| render_comment(item)) {
            scroll_view.render_widget(
                paragraph,
                Rect {
                    x: 0,
                    y,
                    width: area.width - 5,
                    height: comment_height,
                },
            );
            y += comment_height;
            if y >= area.height - 5 {
                break;
            }
        }

        scroll_view.render(area, buf, &mut state.scroll_view_state);
    }
}

fn render_comment<'a>(item: &'a Comment) -> Paragraph<'a> {
    debug!("rendering comment for {}", item.id);
    let elements = html_sanitizer::parse_elements(&item.body);

    let lines = spans(elements)
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();

    Paragraph::new(lines)
        .block(Block::bordered().title(item.by.as_str()))
        .wrap(Wrap { trim: true })
}

fn spans<'a>(elements: Vec<Element<'a>>) -> Vec<Span<'a>> {
    elements
        .into_iter()
        .map(|element| match element {
            Element::Text(s) => Span::styled(s, Style::default()),
            Element::Link(anchor) => {
                let href_attr = anchor.attributes.iter().find(|attr| attr.name == "href");
                if let Some(href_attr) = href_attr {
                    Span::styled(
                        href_attr.value.to_string(),
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    )
                } else {
                    Span::raw("")
                }
            }
            Element::Escaped(c) => Span::styled(c.to_string(), Style::default()),
            Element::Paragraph => Span::styled("\n", Style::default()),
            Element::Code(c) => Span::styled(c, Style::default()),
            Element::Italic(elements) => {
                Span::styled("", Style::default().add_modifier(Modifier::ITALIC))
            }
            Element::Bold(elements) => {
                Span::styled("", Style::default().add_modifier(Modifier::BOLD))
            }
        })
        .collect()
}
