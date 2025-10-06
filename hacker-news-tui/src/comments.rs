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
            // .horizontal_scrollbar_visibility(ScrollbarVisibility::Never)
            .vertical_scrollbar_visibility(ScrollbarVisibility::Always);
        let mut y = 0;

        let paragraph_width = area.width - 5;

        for paragraph in state.comments.iter().map(|item| render_comment(item)) {
            let height = paragraph.line_count(paragraph_width);
            scroll_view.render_widget(
                paragraph,
                Rect {
                    x: 0,
                    y,
                    width: area.width - 5,
                    height: height as u16,
                },
            );
            y += height as u16;
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

    let lines = spans(elements, Style::default())
        .into_iter()
        .collect::<Vec<_>>();

    Paragraph::new(lines)
        .block(Block::bordered().title(item.by.as_str()))
        .wrap(Wrap { trim: true })
}

fn spans<'a>(elements: Vec<Element<'a>>, base_style: Style) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let mut text_spans = Vec::new();

    for element in elements {
        match element {
            Element::Text(s) => {
                text_spans.push(Span::styled(s, base_style));
            }
            Element::Link(anchor) => {
                let href_attr = anchor.attributes.iter().find(|attr| attr.name == "href");
                if let Some(href_attr) = href_attr {
                    text_spans.push(Span::styled(
                        href_attr.value.to_string(),
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    ));
                }
            }
            Element::Escaped(c) => {
                text_spans.push(Span::styled(c.to_string(), base_style));
            }
            Element::Paragraph => {
                lines.extend([Line::raw(""), Line::from(text_spans), Line::raw("")]);
                text_spans = Vec::new();
            }
            Element::Code(c) => {
                text_spans.push(Span::styled(c, Style::default()));
            }
            Element::Italic(elements) => {
                text_spans.extend(sub_spans(
                    elements,
                    base_style.add_modifier(Modifier::ITALIC),
                ));
            }
            Element::Bold(elements) => {
                text_spans.extend(sub_spans(elements, base_style.add_modifier(Modifier::BOLD)));
            }
        }
    }

    if !text_spans.is_empty() {
        lines.push(Line::from(text_spans));
    }

    lines
}

fn sub_spans<'a>(elements: Vec<Element<'a>>, base_style: Style) -> Vec<Span<'a>> {
    let mut text_spans = Vec::new();
    for element in elements {
        match element {
            Element::Text(s) => {
                text_spans.push(Span::styled(s, base_style));
            }
            Element::Link(anchor) => {
                let href_attr = anchor.attributes.iter().find(|attr| attr.name == "href");
                if let Some(href_attr) = href_attr {
                    text_spans.push(Span::styled(
                        href_attr.value.to_string(),
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    ));
                }
            }
            Element::Escaped(c) => {
                text_spans.push(Span::styled(c.to_string(), base_style));
            }
            Element::Paragraph => {}
            Element::Code(c) => {
                text_spans.push(Span::styled(c, Style::default()));
            }
            Element::Italic(elements) => {
                text_spans.extend(sub_spans(
                    elements,
                    base_style.add_modifier(Modifier::ITALIC),
                ));
            }
            Element::Bold(elements) => {
                text_spans.extend(sub_spans(elements, base_style.add_modifier(Modifier::BOLD)));
            }
        }
    }
    text_spans
}
