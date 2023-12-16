use chrono::{DateTime, Utc};
use egui::{RichText, Vec2};
use http_sanitizer::Element;

/// Parse and convert input text into an interator of [`RichText`].
pub fn rich_text(escaped_text: &str) -> impl Iterator<Item = RichText> + '_ {
    let elements = http_sanitizer::as_elements(escaped_text);

    elements.into_iter().flat_map(|element| match element {
        Element::Text(text) => Some(RichText::new(text)),
        Element::Link(link) => link
            .attributes
            .iter()
            .find(|a| a.name == "href")
            .map(|att| RichText::new(att.value.replace("&#x2F;", "/")).underline()),
        Element::Escaped(c) => Some(RichText::new(c)),
        Element::Paragraph => Some(RichText::new("\n\n")),
        // Element::Code(s) => Some(RichText::new(s).code()),
        Element::Code(s) => Some(RichText::new(s).monospace()),
        Element::Italic(s) => Some(RichText::new(s).italics()),
        Element::Bold(s) => Some(RichText::new(s).strong()),
    })
}

/// Render the escaped.
pub fn render_rich_text(escaped_text: &str, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2 { x: 0., y: 0. };
        for text in rich_text(escaped_text) {
            ui.label(text);
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
