use crate::app::AppMsg;
use iced::font::{Style, Weight};
use iced::widget::{rich_text, span};
use iced::{Element, Font};

pub fn render_rich_text(escaped_text: &str) -> Element<'_, AppMsg> {
    let elements = html_sanitizer::parse_elements(escaped_text);

    let spans = elements
        .into_iter()
        .filter_map(|element| match element {
            html_sanitizer::Element::Text(text) => Some(span(text)),
            html_sanitizer::Element::Link(link) => link
                .attributes
                .into_iter()
                .find(|attr| attr.name == "href")
                .map(move |attr| {
                    if link.children.is_empty() {
                        span(attr.value.clone()).link(AppMsg::OpenLink(attr.value))
                    } else {
                        span(link.children).link(AppMsg::OpenLink(attr.value))
                    }
                }),
            html_sanitizer::Element::Escaped(text) => Some(span(text)),
            html_sanitizer::Element::Paragraph => Some(span("\n\n")),
            html_sanitizer::Element::Code(text) => Some(span(text).font(Font::MONOSPACE)),
            html_sanitizer::Element::Italic(text) => Some(span(text).font(Font {
                style: Style::Italic,
                ..Default::default()
            })),
            html_sanitizer::Element::Bold(text) => Some(span(text).font(Font {
                weight: Weight::Bold,
                ..Default::default()
            })),
        })
        .collect::<Vec<_>>();

    rich_text(spans).into()
}
