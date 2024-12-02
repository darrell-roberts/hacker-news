use crate::app::AppMsg;
use iced::font::{Style, Weight};
use iced::widget::span;
use iced::widget::text::Span;
use iced::{Color, Font};
use std::ops::Not;

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
            html_sanitizer::Element::Link(link) => spans.extend(
                link.attributes
                    .into_iter()
                    .find(|attr| attr.name == "href")
                    .map(move |attr| {
                        if link.children.is_empty() {
                            span(attr.value.clone()).link(AppMsg::OpenLink {
                                url: attr.value,
                                item_id: 0,
                            })
                        } else {
                            span(link.children.clone()).link(AppMsg::OpenLink {
                                url: attr.value,
                                item_id: 0,
                            })
                        }
                    })
                    .into_iter(),
            ),
            html_sanitizer::Element::Escaped(text) => spans.push(span(text)),
            html_sanitizer::Element::Paragraph => {
                if oneline {
                    break;
                }
                spans.push(span("\n\n"))
            }
            html_sanitizer::Element::Code(text) => {
                // spans.extend(split_search(text, search).map(|s| s.font(Font::MONOSPACE)))
                spans.push(span(text).font(Font::MONOSPACE));
            }
            html_sanitizer::Element::Italic(text) => {
                // spans.extend(split_search(text, search).map(|s| {
                //     s.font(Font {
                //         style: Style::Italic,
                //         ..Default::default()
                //     })
                // }))
                spans.push(span(text).font(Font {
                    style: Style::Italic,
                    ..Default::default()
                }));
            }
            html_sanitizer::Element::Bold(text) => {
                // spans.extend(split_search(text, search).map(|s| {
                //     s.font(Font {
                //         weight: Weight::Bold,
                //         ..Default::default()
                //     })
                // }))
                spans.push(span(text).font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                }));
            }
        }
    }

    spans
    // rich_text(spans).into()
}

// fn split_search<'a>(text: &'a str, search: Option<&str>) -> Vec<Span<'a, AppMsg>> {
//     match search {
//         Some(s) => {
//             let mut r = Vec::new();
//             let mut last_index = 0;
//             for (index, matched) in text.match_indices(s) {
//                 let (unmatched, _) = text.split_at(index);
//                 if !unmatched.is_empty() {
//                     let segment = &unmatched[last_index..];
//                     r.push(span(segment));
//                 }
//                 r.push(
//                     span(matched)
//                         .color(Color::BLACK)
//                         .background(Color::from_rgb8(255, 255, 0)),
//                 );
//                 last_index = index + s.len();
//             }
//             r.push(span(&text[last_index..]));
//             r
//         }
//         None => vec![span(text)],
//     }
// }

pub struct SearchSpanIter<'a> {
    last_index: usize,
    search: Option<&'a str>,
    text: &'a str,
    matcher: Option<Box<dyn Iterator<Item = (usize, &'a str)> + 'a>>,
    finished: bool,
    next_match: Option<Span<'a, AppMsg>>,
}

impl<'a> SearchSpanIter<'a> {
    pub fn new(text: &'a str, search: Option<&'a str>) -> Self {
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

impl<'a> Iterator for SearchSpanIter<'a> {
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

/*
#[cfg(test)]
mod test {
    use crate::richtext::split_search;

    #[test]
    fn test_split_search() {
        let test_phrase = "The quick brown fox jumped over the lazy dogs. The quick brown fox jumped over the lazy dogs.";

        let r = split_search(test_phrase, Some("the"));
        dbg!(r);
    }
}
*/
