use crate::app::AppMsg;
use iced::font::{Style, Weight};
use iced::widget::text::Span;
use iced::widget::{rich_text, span};
use iced::{Background, Color, Element, Font};
use std::borrow::Cow;

pub fn render_rich_text<'a>(escaped_text: &'a str, search: Option<&'a str>) -> Element<'a, AppMsg> {
    let elements = html_sanitizer::parse_elements(escaped_text);

    let mut spans = Vec::new();

    for e in elements {
        match e {
            html_sanitizer::Element::Text(text) => spans.extend(search_span_slit(text, search)),
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
            html_sanitizer::Element::Paragraph => spans.push(span("\n\n")),
            html_sanitizer::Element::Code(text) => {
                spans.extend(search_span_slit(text, search).map(|s| s.font(Font::MONOSPACE)))
            }
            html_sanitizer::Element::Italic(text) => {
                spans.extend(search_span_slit(text, search).map(|s| {
                    s.font(Font {
                        style: Style::Italic,
                        ..Default::default()
                    })
                }))
            }
            html_sanitizer::Element::Bold(text) => {
                spans.extend(search_span_slit(text, search).map(|s| {
                    s.font(Font {
                        weight: Weight::Bold,
                        ..Default::default()
                    })
                }))
            }
        }
    }

    rich_text(spans).into()
}

fn search_span_slit<'a>(
    text: impl Into<Cow<'a, str>>,
    search: Option<&'a str>,
) -> impl Iterator<Item = Span<'a, AppMsg>> + 'a {
    let t = text.into();
    t.into_owned().split_whitespace().map(move |word| {
        if search
            .map(|s| s.to_lowercase().eq(&word.to_lowercase()))
            .unwrap_or(false)
        {
            span(word).background(Background::Color(Color::from_rgb8(255, 255, 0)))
        } else {
            span(word)
        }
    })
}

// fn search_span_split_2(
//     text: String,
//     search: Option<&str>,
// ) -> impl Iterator<Item = Span<'_, AppMsg>> + '_ {
//     text.split_whitespace().map(move |word| {
//         if search
//             .map(|s| s.to_lowercase().eq(&word.to_lowercase()))
//             .unwrap_or(false)
//         {
//             span(word.to_owned()).background(Background::Color(Color::from_rgb8(255, 255, 0)))
//         } else {
//             span(word.to_owned())
//         }
//     })
// }
