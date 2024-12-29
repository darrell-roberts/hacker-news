//! Renders rich text from a simplified html string. Allows creating spans
//! for search matches so search strings can be highlighted.
use crate::app::AppMsg;
use html_sanitizer::Anchor;
use iced::font::{Style, Weight};
use iced::widget::span;
use iced::widget::text::Span;
use iced::{Color, Font};
use std::ops::Not;

/// Render a simplified html string into `Span`s for a `RichText` widget.
pub fn render_rich_text<'a>(
    escaped_text: &'a str,
    search: Option<&'a str>,
    oneline: bool,
) -> Vec<Span<'a, AppMsg>> {
    let elements = html_sanitizer::parse_elements(escaped_text);
    let mut spans = Vec::new();

    for e in elements {
        match e {
            html_sanitizer::Element::Text(text) => spans.extend(SearchSpanIter::new(text, search)),
            html_sanitizer::Element::Link(link) => spans.extend(anchor_spans(link)),
            html_sanitizer::Element::Escaped(text) => spans.push(span(text)),
            html_sanitizer::Element::Paragraph => {
                if oneline {
                    break;
                }
                spans.push(span("\n\n"))
            }
            html_sanitizer::Element::Code(text) => {
                spans.extend(split_search(text, search, |span| {
                    span.font(Font::MONOSPACE)
                }));
            }
            // We can have one level nesting here.
            html_sanitizer::Element::Italic(nested) => {
                for el in nested {
                    match el {
                        html_sanitizer::Element::Text(text) => spans.extend(
                            SearchSpanIter::new(text, search)
                                .map(|s| s.font(Font::default().italic())),
                        ),
                        html_sanitizer::Element::Link(link) => spans.extend(anchor_spans(link)),
                        html_sanitizer::Element::Escaped(text) => {
                            spans.push(span(text).font(Font::default().italic()))
                        }
                        html_sanitizer::Element::Paragraph => spans.push(span("\n\n")),
                        html_sanitizer::Element::Bold(nested) => {
                            for el in nested {
                                match el {
                                    html_sanitizer::Element::Text(text) => spans.extend(
                                        SearchSpanIter::new(text, search)
                                            .map(|s| s.font(Font::default().bold().italic())),
                                    ),
                                    html_sanitizer::Element::Link(link) => {
                                        spans.extend(anchor_spans(link))
                                    }
                                    html_sanitizer::Element::Escaped(text) => {
                                        spans.push(span(text).font(Font::default().italic().bold()))
                                    }
                                    html_sanitizer::Element::Paragraph => spans.push(span("\n\n")),
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            // We can have one level nesting here.
            html_sanitizer::Element::Bold(inner) => {
                for el in inner {
                    match el {
                        html_sanitizer::Element::Text(text) => spans.extend(
                            SearchSpanIter::new(text, search)
                                .map(|s| s.font(Font::default().bold())),
                        ),
                        html_sanitizer::Element::Link(link) => spans.extend(anchor_spans(link)),
                        html_sanitizer::Element::Escaped(text) => {
                            spans.push(span(text).font(Font::default().bold()))
                        }
                        html_sanitizer::Element::Paragraph => spans.push(span("\n\n")),
                        html_sanitizer::Element::Italic(nested) => {
                            for el in nested {
                                match el {
                                    html_sanitizer::Element::Text(text) => spans.extend(
                                        SearchSpanIter::new(text, search)
                                            .map(|s| s.font(Font::default().bold().italic())),
                                    ),
                                    html_sanitizer::Element::Link(link) => {
                                        spans.extend(anchor_spans(link))
                                    }
                                    html_sanitizer::Element::Escaped(text) => {
                                        spans.push(span(text).font(Font::default().italic().bold()))
                                    }
                                    html_sanitizer::Element::Paragraph => spans.push(span("\n\n")),
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    spans
}

fn anchor_spans(link: Anchor<'_>) -> impl Iterator<Item = Span<'_, AppMsg>> {
    let children = link.children;
    let msg = |url| AppMsg::OpenLink { url, item_id: 0 };

    link.attributes
        .into_iter()
        .find_map(|attr| (attr.name == "href").then_some(attr.value))
        .map(move |url| {
            if children.is_empty() {
                span(url.clone()).link(msg(url))
            } else {
                span(children).link(msg(url))
            }
        })
        .into_iter()
}

trait FontExt {
    fn bold(self) -> Self;
    fn italic(self) -> Self;
}

impl FontExt for Font {
    fn bold(self) -> Self {
        Self {
            weight: Weight::Bold,
            ..self
        }
    }

    fn italic(self) -> Self {
        Self {
            style: Style::Italic,
            ..self
        }
    }
}

/// Split an owned string into multiple owned spans.
fn split_search(
    text: String,
    search: Option<&str>,
    update_span: impl Fn(Span<'_, AppMsg>) -> Span<'_, AppMsg>,
) -> Vec<Span<'_, AppMsg>> {
    match search {
        Some(s) => {
            let mut spans = Vec::new();
            let mut last_index = 0;

            for (index, matched) in text.match_indices(s) {
                let (unmatched, _) = text.split_at(index);
                if !unmatched.is_empty() {
                    let segment = &unmatched[last_index..];
                    spans.push(update_span(span(segment.to_owned())));
                }
                spans.push(update_span(
                    span(matched.to_owned())
                        .color(Color::BLACK)
                        .background(Color::from_rgb8(255, 255, 0)),
                ));
                last_index = index + s.len();
            }

            let remaining = &text[last_index..];
            if !remaining.is_empty() {
                spans.push(update_span(span(remaining.to_owned())));
            }

            spans
        }
        None => vec![update_span(span(text))],
    }
}

/// Yield multiple spans from a single str reference.
pub struct SearchSpanIter<'a, 'b> {
    last_index: usize,
    search: Option<&'b str>,
    text: &'a str,
    matcher: Option<Box<dyn Iterator<Item = (usize, &'a str)> + 'a>>,
    finished: bool,
    next_match: Option<Span<'a, AppMsg>>,
}

impl<'a, 'b> SearchSpanIter<'a, 'b> {
    pub fn new(text: &'a str, search: Option<&'b str>) -> Self {
        Self {
            last_index: 0,
            search,
            text,
            matcher: None,
            finished: false,
            next_match: None,
        }
    }
}

impl<'a, 'b: 'a> Iterator for SearchSpanIter<'a, 'b> {
    type Item = Span<'a, AppMsg>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        if let Some(s) = self.next_match.take() {
            return Some(s);
        }

        match self.search {
            Some(search) => {
                if self.matcher.is_none() {
                    self.matcher = Some(Box::new(self.text.match_indices(search)));
                }

                if let Some((index, matched)) = self.matcher.as_mut()?.next() {
                    let (unmatched, _) = self.text.split_at(index);

                    let segment: Option<Span<'a, AppMsg>> = unmatched
                        .is_empty()
                        .not()
                        .then(|| span(&unmatched[self.last_index..]));

                    let next_match = Some(
                        span(matched)
                            .color(Color::BLACK)
                            .background(Color::from_rgb8(255, 255, 0)),
                    );
                    self.last_index = index + search.len();

                    if segment.is_some() {
                        self.next_match = next_match;
                        segment
                    } else {
                        next_match
                    }
                } else {
                    self.finished = true;
                    Some(span(&self.text[self.last_index..]))
                }
            }
            None => {
                self.finished = true;
                Some(span(self.text))
            }
        }
    }
}
