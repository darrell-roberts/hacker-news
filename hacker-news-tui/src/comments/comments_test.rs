use html_sanitizer::parse_elements;
use ratatui::style::Style;

use crate::comments::spans;

#[test]
fn test_mark_up() {
    let test = "&quot;Samsung Family Hub™ for 2025 Update Elevates the Smart Home Ecosystem\nThe software update includes a more unified user experience across connected devices, enhancements to AI Vision Inside™, expanded Knox Security and more&quot;<p>In plain English now, Samgung will put advertising on your face but mostly important is that they will know and sell the data about what you buy.<p>They call it Smart Home. Yeah, &quot;smart&quot;. They are the smart ones, not those who buy this **.";

    let elements = parse_elements(test);

    dbg!(&elements);

    let _ = spans(elements, Style::default(), None);
}
