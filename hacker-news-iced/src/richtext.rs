//! Renders rich text from a simplified html string. Allows creating spans
//! for search matches so search strings can be highlighted.
use crate::{ROBOTO_FONT, ROBOTO_MONO, common::FontExt};
use html_sanitizer::Anchor;
use iced::{
    Color,
    widget::{span, text::Span},
};
use std::str::MatchIndices;

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
    text: &'a str,
    search_len: usize,
    matcher: Option<MatchIndices<'a, &'b str>>,
    next_match: Option<Span<'a, Link>>,
}

impl<'a, 'b, Link> SearchSpanIter<'a, 'b, Link> {
    pub fn new(text: &'a str, search: Option<&'b str>) -> Self {
        Self {
            last_index: 0,
            search_len: search.map_or(0, str::len),
            text,
            matcher: search.map(|s| text.match_indices(s)),
            next_match: None,
        }
    }
}

impl<'a, 'b: 'a, Link> Iterator for SearchSpanIter<'a, 'b, Link> {
    type Item = Span<'a, Link>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.next_match.take() {
            return Some(s);
        }

        if let Some((index, matched)) = self.matcher.as_mut().and_then(|m| m.next()) {
            let before = &self.text[self.last_index..index];
            self.last_index = index + self.search_len;

            let hit = span(matched)
                .color(Color::BLACK)
                .background(Color::from_rgb8(255, 255, 0));

            if before.is_empty() {
                Some(hit)
            } else {
                self.next_match = Some(hit);
                Some(span(before))
            }
        } else {
            let rest = &self.text[self.last_index..];
            self.last_index = self.text.len();
            (!rest.is_empty()).then(|| span(rest))
        }
    }
}
