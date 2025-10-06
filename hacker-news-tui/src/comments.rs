//! Comments view widget.
use hacker_news_search::api::Comment;
use html_sanitizer::Element;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect, Size},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, StatefulWidget, Widget, Wrap},
};
use tui_scrollview::ScrollViewState;

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
pub struct CommentsWidget<'a> {
    article_title: &'a str,
}

impl<'a> CommentsWidget<'a> {
    pub fn new(article_title: &'a str) -> Self {
        Self { article_title }
    }
}

impl<'a> StatefulWidget for &mut CommentsWidget<'a> {
    type State = CommentState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let [title, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        Line::raw(self.article_title).render(title, buf);

        let paragraph_widgets = state
            .comments
            .iter()
            .map(render_comment)
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

        scroll_view.render(body, buf, &mut state.scroll_view_state);
    }
}

fn render_comment<'a>(item: &'a Comment) -> Paragraph<'a> {
    let elements = html_sanitizer::parse_elements(&item.body);

    let lines = spans(elements, Style::default())
        .into_iter()
        .collect::<Vec<_>>();

    Paragraph::new(lines)
        .block(
            Block::bordered()
                .title_bottom(format!("by {} [{}]", item.by.as_str(), item.kids.len()))
                .title_alignment(Alignment::Right),
        )
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

    lines.push(Line::from(""));

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
