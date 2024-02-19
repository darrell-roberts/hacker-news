use crate::app::AppMsg;
use iced::font::{Style, Weight};
use iced::widget::text;
use iced::{Element, Font};

pub fn render_rich_text(escaped_text: &str) -> Vec<Element<AppMsg>> {
    let elements = html_sanitizer::parse_elements(escaped_text);

    let mut text_string = String::new();
    let mut render_elements = Vec::new();

    let add_text = |es: &mut Vec<_>, ts: &mut String| {
        if !ts.is_empty() {
            es.push(Element::from(text(&ts)));
            *ts = String::new();
        }
    };

    for element in elements {
        match element {
            html_sanitizer::Element::Text(text) => {
                text_string.push_str(text);
            }
            html_sanitizer::Element::Link(link) => {
                // add_text(&mut render_elements, &mut text_string);
                if let Some(attr) = link.attributes.into_iter().find(|attr| attr.name == "href") {
                    if link.children.is_empty() {
                        text_string.push_str(&attr.value);
                    } else {
                        text_string.push_str(&link.children);
                    }
                }
            }
            html_sanitizer::Element::Escaped(c) => {
                text_string.push(c);
            }
            html_sanitizer::Element::Paragraph => {
                text_string.push_str("\n\n");
            }
            html_sanitizer::Element::Code(s) => {
                add_text(&mut render_elements, &mut text_string);
                render_elements.push(Element::from(text(s).font(Font::MONOSPACE)))
            }
            html_sanitizer::Element::Italic(s) => {
                // let row = row![
                //     text(&text_string),
                //     text(s).font(Font {
                //         style: Style::Italic,
                //         ..Default::default()
                //     })
                // ];
                // text_string = String::new();
                // render_elements.push(Element::from(row));

                render_elements.push(Element::from(text(&text_string)));
                render_elements.push(Element::from(text(s).font(Font {
                    style: Style::Italic,
                    ..Default::default()
                })));
                // add_text(&mut render_elements, &mut text_string);

                // render_elements.push(Element::from(Row::new().push(text(s).font(Font {
                //     style: Style::Italic,
                //     ..Default::default()
                // }))));
            }
            html_sanitizer::Element::Bold(s) => {
                // let row = row![
                //     text(&text_string),
                //     text(s).font(Font {
                //         weight: Weight::Bold,
                //         ..Default::default()
                //     })
                // ];
                // text_string = String::new();
                // render_elements.push(Element::from(row));

                render_elements.push(Element::from(text(&text_string)));
                render_elements.push(Element::from(text(s).font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                })));
                // add_text(&mut render_elements, &mut text_string);
                // render_elements.push(Element::from(Row::new().push(text(s).font(Font {
                //     weight: Weight::Bold,
                //     ..Default::default()
                // }))));
            }
        }
    }

    add_text(&mut render_elements, &mut text_string);

    render_elements
    // text(text_string).into()
}
