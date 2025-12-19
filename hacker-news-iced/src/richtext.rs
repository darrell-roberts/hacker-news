//! Renders rich text from a simplified html string. Allows creating spans
//! for search matches so search strings can be highlighted.
use crate::{common::FontExt, ROBOTO_FONT, ROBOTO_MONO};
use html_sanitizer::Anchor;
use iced::{
    widget::{span, text::Span},
    Color,
};
use std::ops::Not;

/// Render a simplified html string into `Span`s for a `RichText` widget.
pub fn render_rich_text<'a>(
    escaped_text: &'a str,
    search: Option<&'a str>,
    oneline: bool,
) -> Vec<Span<'a, String>> {
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
                spans.extend(split_search(text, search, |span| span.font(ROBOTO_MONO)));
            }
            // We can have one level nesting here.
            html_sanitizer::Element::Italic(nested) => {
                for el in nested {
                    match el {
                        html_sanitizer::Element::Text(text) => spans.extend(
                            SearchSpanIter::new(text, search).map(|s| s.font(ROBOTO_FONT.italic())),
                        ),
                        html_sanitizer::Element::Link(link) => spans.extend(anchor_spans(link)),
                        html_sanitizer::Element::Escaped(text) => {
                            spans.push(span(text).font(ROBOTO_FONT.italic()))
                        }
                        html_sanitizer::Element::Paragraph => spans.push(span("\n\n")),
                        html_sanitizer::Element::Bold(nested) => {
                            for el in nested {
                                match el {
                                    html_sanitizer::Element::Text(text) => spans.extend(
                                        SearchSpanIter::new(text, search)
                                            .map(|s| s.font(ROBOTO_FONT.bold().italic())),
                                    ),
                                    html_sanitizer::Element::Link(link) => {
                                        spans.extend(anchor_spans(link))
                                    }
                                    html_sanitizer::Element::Escaped(text) => {
                                        spans.push(span(text).font(ROBOTO_FONT.italic().bold()))
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
                            SearchSpanIter::new(text, search).map(|s| s.font(ROBOTO_FONT.bold())),
                        ),
                        html_sanitizer::Element::Link(link) => spans.extend(anchor_spans(link)),
                        html_sanitizer::Element::Escaped(text) => {
                            spans.push(span(text).font(ROBOTO_FONT.bold()))
                        }
                        html_sanitizer::Element::Paragraph => spans.push(span("\n\n")),
                        html_sanitizer::Element::Italic(nested) => {
                            for el in nested {
                                match el {
                                    html_sanitizer::Element::Text(text) => spans.extend(
                                        SearchSpanIter::new(text, search)
                                            .map(|s| s.font(ROBOTO_FONT.bold().italic())),
                                    ),
                                    html_sanitizer::Element::Link(link) => {
                                        spans.extend(anchor_spans(link))
                                    }
                                    html_sanitizer::Element::Escaped(text) => {
                                        spans.push(span(text).font(ROBOTO_FONT.italic().bold()))
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

fn anchor_spans(link: Anchor<'_>) -> impl Iterator<Item = Span<'_, String>> {
    let children = link.children;

    link.attributes
        .into_iter()
        .find_map(|attr| (attr.name == "href").then_some(attr.value))
        .map(move |url| {
            if children.is_empty() {
                span(url.clone()).link(url)
            } else {
                span(children).link(url)
            }
        })
        .into_iter()
}

/// Split an owned string into multiple owned spans.
fn split_search<Link>(
    text: String,
    search: Option<&str>,
    update_span: impl Fn(Span<'_, Link>) -> Span<'_, Link>,
) -> Vec<Span<'_, Link>> {
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
pub struct SearchSpanIter<'a, 'b, Link> {
    last_index: usize,
    search: Option<&'b str>,
    text: &'a str,
    matcher: Option<Box<dyn Iterator<Item = (usize, &'a str)> + 'a>>,
    finished: bool,
    next_match: Option<Span<'a, Link>>,
}

impl<'a, 'b, Link> SearchSpanIter<'a, 'b, Link> {
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

impl<'a, 'b: 'a, Link> Iterator for SearchSpanIter<'a, 'b, Link> {
    type Item = Span<'a, Link>;

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

                    let segment: Option<Span<'a, Link>> = unmatched
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
