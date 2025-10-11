//! Comments view widget.
use std::borrow::Cow;

use hacker_news_search::api::{AgeLabel, Comment};
use html_sanitizer::Element;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect, Size},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Widget, Wrap},
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
    pub child_stack: Vec<u64>,
    pub page_height: u16,
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
}

fn render_comment<'a>(item: &'a Comment, selected: bool) -> Paragraph<'a> {
    let elements = html_sanitizer::parse_elements(&item.body);

    let lines = spans(elements, Style::default())
        .into_iter()
        .collect::<Vec<_>>();

    let title = Line::from_iter([
        Cow::Borrowed("by "),
        Cow::Borrowed(item.by.as_str()),
        Cow::Borrowed(" "),
        Cow::Owned(item.age_label().unwrap_or_default()),
        if item.kids.is_empty() {
            Cow::Borrowed("")
        } else {
            Cow::Owned(format!(" [{}]", item.kids.len()))
        },
    ]);

    Paragraph::new(lines)
        .block(
            Block::bordered()
                .border_style(if selected {
                    Style::new().green().on_black()
                } else {
                    Style::new()
                })
                .border_type(if selected {
                    BorderType::QuadrantInside
                } else {
                    BorderType::Plain
                })
                .title_bottom(title)
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
                if lines.is_empty() {
                    lines.extend([Line::from(text_spans), Line::raw("")]);
                } else {
                    lines.extend([Line::raw(""), Line::from(text_spans), Line::raw("")]);
                }
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
