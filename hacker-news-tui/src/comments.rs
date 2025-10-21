//! Comments view widget.
use hacker_news_search::{
    SearchContext,
    api::{AgeLabel, Comment},
};
use html_sanitizer::{Anchor, Element};
use log::error;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect, Size},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, StatefulWidget, Widget, Wrap},
};
use std::sync::{Arc, RwLock};
use tui_scrollview::ScrollViewState;

#[derive(Default, Debug)]
pub struct CommentStack {
    pub parent_id: u64,
    pub offset: usize,
    pub scroll_view_state: ScrollViewState,
}

#[derive(Default)]
pub struct CommentState {
    pub parent_id: u64,
    pub limit: usize,
    pub offset: usize,
    pub viewing: Option<usize>,
    pub comments: Vec<Comment>,
    pub total_comments: usize,
    pub scroll_view_state: ScrollViewState,
    pub child_stack: Vec<CommentStack>,
    pub page_height: u16,
}

impl CommentState {
    pub fn page_forward(&mut self, search_context: Arc<RwLock<SearchContext>>) {
        self.viewing = None;
        self.update_offset(self.offset.saturating_add(10));
        self.update_comments(search_context);
        self.scroll_view_state.scroll_to_top();
    }

    pub fn page_back(&mut self, search_context: Arc<RwLock<SearchContext>>) {
        self.viewing = None;
        self.update_offset(self.offset.saturating_sub(10));
        self.update_comments(search_context);
    }

    fn update_offset(&mut self, next_offset: usize) {
        if next_offset / 10 < self.total_pages() {
            self.offset = next_offset;
        }
    }

    fn update_comments(&mut self, search_context: Arc<RwLock<SearchContext>>) {
        let result = search_context
            .read()
            .unwrap()
            .comments(self.parent_id, 10, self.offset);
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

    fn total_pages(&self) -> usize {
        let remainder = self.total_comments % 10;
        self.total_comments / 10 + if remainder > 0 { 1 } else { 0 }
    }

    fn selected_page(&self) -> usize {
        if self.offset == 0 {
            1
        } else {
            self.offset / 10 + 1
        }
    }
}

/// Comments view widget.
#[derive(Default)]
pub struct CommentsWidget<'a> {
    article_title: &'a str,
    article_body: Option<&'a str>,
}

impl<'a> CommentsWidget<'a> {
    /// Create a comment view with a a story title and body.
    pub fn new(article_title: &'a str, body: Option<&'a str>) -> Self {
        Self {
            article_title,
            article_body: body,
        }
    }
}

impl<'a> StatefulWidget for &mut CommentsWidget<'a> {
    type State = CommentState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        // Split layout into title scrollable content and pagination.
        let [title_area, content_area, page_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        // Article title
        Line::styled(self.article_title, Style::new().bold()).render(title_area, buf);

        // Optional article body
        let body_paragraph = self
            .article_body
            .filter(|body| !body.is_empty())
            .map(|body| {
                let elements = html_sanitizer::parse_elements(body);
                Paragraph::new(spans(elements, Style::default())).wrap(Wrap { trim: false })
            });

        // Comments
        render_comments(buf, state, content_area, body_paragraph);

        // Pagination pages
        if state.total_comments > 0 {
            let selected_page = state.selected_page();
            let spans = (1..=state.total_pages()).map(|page| {
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
    }
}

fn render_comments(
    buf: &mut Buffer,
    state: &mut CommentState,
    body: Rect,
    article_body: Option<Paragraph<'_>>,
) {
    let paragraph_widgets = article_body
        .into_iter()
        .chain(
            state
                .comments
                .iter()
                .zip(0..)
                .map(|(item, index)| render_comment(item, state.viewing == Some(index))),
        )
        .collect::<Vec<_>>();

    let scroll_view_height: u16 = paragraph_widgets
        .iter()
        .map(|p| p.line_count(buf.area.width))
        .sum::<usize>() as u16;

    let width = if buf.area.height < scroll_view_height {
        buf.area.width - 1
    } else {
        buf.area.width
    };

    let mut scroll_view = tui_scrollview::ScrollView::new(Size::new(width, scroll_view_height));
    let mut y = 0;

    for paragraph in paragraph_widgets {
        let height = paragraph.line_count(width);
        scroll_view.render_widget(
            paragraph,
            Rect {
                x: 0,
                y,
                width,
                height: height as u16,
            },
        );
        y += height as u16;
    }

    state.page_height = body.height;
    scroll_view.render(body, buf, &mut state.scroll_view_state);
}

pub fn render_comment<'a>(item: &'a Comment, selected: bool) -> Paragraph<'a> {
    let elements = html_sanitizer::parse_elements(&item.body);

    let lines = spans(elements, Style::default())
        .into_iter()
        .collect::<Vec<_>>();

    let title = Line::from_iter([
        Span::raw("by "),
        Span::raw(item.by.as_str()),
        Span::raw(" "),
        Span::raw(item.age_label().unwrap_or_default()),
        if item.kids.is_empty() {
            Span::raw("")
        } else {
            Span::raw(format!(" [{}]", item.kids.len()))
        },
    ])
    .style(Style::new().italic());

    Paragraph::new(lines)
        .block(
            Block::bordered()
                .border_type(if selected {
                    BorderType::Thick
                } else {
                    BorderType::Rounded
                })
                .title_bottom(title)
                .title_alignment(Alignment::Right),
        )
        .style(if selected {
            Style::new().white().on_dark_gray().bold()
        } else {
            Style::default()
        })
        .wrap(Wrap { trim: false })
}

fn spans<'a>(elements: Vec<Element<'a>>, base_style: Style) -> Vec<Line<'a>> {
    let mut lines: Vec<Line<'_>> = Vec::new();
    let mut text_spans = Vec::new();

    let mut element_iter = elements.into_iter().peekable();
    let mut append_last_line = false;
    let mut count = 0;

    while let Some(element) = element_iter.next() {
        match element {
            Element::Text(s) => {
                let multi_line = s.lines().count() > 1;

                if multi_line {
                    if append_last_line
                        && let Some(last_line) = lines.last_mut()
                        && let Some(next_line) = s.lines().next()
                    {
                        last_line.push_span(Span::styled(next_line, base_style));
                    } else {
                        lines.push(Line::from(text_spans));
                        text_spans = Vec::new();
                    }
                    lines.extend(
                        s.lines()
                            .skip(if append_last_line { 1 } else { 0 })
                            .map(|line| Line::from(Span::styled(line, base_style))),
                    );
                } else if append_last_line && let Some(last_line) = lines.last_mut() {
                    last_line.push_span(Span::styled(s, base_style));
                } else {
                    text_spans.push(Span::styled(s, base_style));
                }

                let last_append_last_line = append_last_line;

                // Look ahead to see if we need to append to last line.
                append_last_line = matches!(
                    element_iter.peek(),
                    Some(
                        Element::Escaped(_)
                            | Element::Link(_)
                            | Element::Bold(_)
                            | Element::Italic(_)
                    )
                );

                if !last_append_last_line && append_last_line && !text_spans.is_empty() {
                    lines.push(Line::from(text_spans));
                    text_spans = Vec::new();
                }
            }
            Element::Link(anchor) => {
                if append_last_line && let Some(last_line) = lines.last_mut() {
                    if let Some(span) = maybe_span(anchor) {
                        last_line.push_span(span);
                        if count == 0 {
                            append_last_line = true;
                        }
                    }
                } else {
                    let span = maybe_span(anchor);
                    if span.is_some() {
                        append_last_line = true;
                    }
                    text_spans.extend(span);
                }
            }
            Element::Escaped(c) => {
                if append_last_line && let Some(last_line) = lines.last_mut() {
                    last_line.push_span(Span::styled(c.to_string(), base_style));
                } else {
                    text_spans.push(Span::styled(c.to_string(), base_style));
                }
            }
            Element::Paragraph => {
                lines.push(Line::from(text_spans));
                text_spans = Vec::new();
                append_last_line = false;
            }
            Element::Code(c) => {
                if !text_spans.is_empty() {
                    lines.push(Line::from(text_spans));
                    text_spans = Vec::new();
                }
                lines.extend(c.lines().map(|line| Line::raw(line.to_owned())));
                append_last_line = false;
            }
            Element::Italic(elements) => {
                if append_last_line && let Some(last_line) = lines.last_mut() {
                    last_line.extend(sub_spans(
                        elements,
                        base_style.add_modifier(Modifier::ITALIC),
                    ));
                } else {
                    text_spans.extend(sub_spans(
                        elements,
                        base_style.add_modifier(Modifier::ITALIC),
                    ));
                }
            }
            Element::Bold(elements) => {
                if append_last_line && let Some(last_line) = lines.last_mut() {
                    last_line.extend(sub_spans(elements, base_style.add_modifier(Modifier::BOLD)));
                } else {
                    text_spans.extend(sub_spans(elements, base_style.add_modifier(Modifier::BOLD)));
                }
            }
        }
        count += 1;
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
            Element::Escaped(c) => {
                text_spans.push(Span::styled(c.to_string(), base_style));
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
            // Sub elements won't have this
            Element::Paragraph | Element::Code(_) | Element::Link(_) => {}
        }
    }
    text_spans
}

fn maybe_span(anchor: Anchor<'_>) -> Option<Span<'_>> {
    anchor
        .attributes
        .into_iter()
        .find(|attr| attr.name == "href")
        .map(|href_attr| {
            Span::styled(
                href_attr.value,
                Style::default().add_modifier(Modifier::UNDERLINED),
            )
        })
}
