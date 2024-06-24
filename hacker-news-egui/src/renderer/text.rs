use chrono::{DateTime, Utc};
use egui::{Color32, RichText, Vec2};
use html_sanitizer::Element;

use crate::{app::HackerNewsApp, event::Event};

/// Render html escaped text into the Ui.
pub fn render_rich_text(app_state: &HackerNewsApp, escaped_text: &str, ui: &mut egui::Ui) {
    let elements = html_sanitizer::parse_elements(escaped_text);

    ui.horizontal_wrapped(|ui| {
        ui.style_mut().visuals.hyperlink_color = Color32::DARK_RED;
        ui.spacing_mut().item_spacing = Vec2 { x: 0., y: 0. };

        let mut text_string = String::new();

        let render_text = |ui: &mut egui::Ui, text: &mut String| {
            if !text.is_empty() {
                ui.label(text.as_str());
                text.clear();
            }
        };

        for element in elements {
            match element {
                Element::Text(text) => {
                    text_string.push_str(text);
                }
                Element::Link(link) => {
                    render_text(ui, &mut text_string);
                    if let Some(text) = link
                        .attributes
                        .iter()
                        .find(|a| a.name == "href")
                        .map(|att| att.value.as_str())
                    {
                        let name = if link.children.is_empty() {
                            text
                        } else {
                            link.children.as_str()
                        };
                        let hyper_link = ui.hyperlink_to(name, text).on_hover_text(text);
                        if hyper_link.secondary_clicked() {
                            app_state.emit(Event::CopyToClipboard(name.to_string()));
                        }
                    }
                }
                Element::Escaped(c) => {
                    text_string.push(c);
                }
                Element::Paragraph => {
                    text_string.push_str("\n\n");
                }
                Element::Code(text) => {
                    render_text(ui, &mut text_string);
                    ui.label(RichText::new(text).monospace());
                }
                Element::Italic(text) => {
                    render_text(ui, &mut text_string);
                    ui.label(RichText::new(text).italics());
                }
                Element::Bold(text) => {
                    render_text(ui, &mut text_string);
                    ui.label(RichText::new(text).heading());
                }
            }
        }

        if !text_string.is_empty() {
            ui.label(&text_string);
        }
    });
}

/// Extract the duration from a UNIX time and convert duration into a human
/// friendly sentence.
pub fn parse_date(time: u64) -> Option<String> {
    let duration = DateTime::<Utc>::from_timestamp(time as i64, 0).map(|then| Utc::now() - then)?;

    let hours = duration.num_hours();
    let minutes = duration.num_minutes();
    let days = duration.num_days();

    match (days, hours, minutes) {
        (0, 0, 1) => "1 minute ago".to_string(),
        (0, 0, m) => format!("{m} minutes ago"),
        (0, 1, _) => "1 hour ago".to_string(),
        (0, h, _) => format!("{h} hours ago"),
        (1, _, _) => "1 day ago".to_string(),
        (d, _, _) => format!("{d} days ago"),
    }
    .into()
}
