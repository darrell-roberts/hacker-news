//! Re-usable styles
use ratatui::style::{Color, Style, Stylize as _};

/// An item is selected
pub fn selected_style() -> Style {
    Style::new()
        .fg(Color::from_u32(0xe6e6e6))
        .bg(Color::from_u32(0x990000))
        .bold()
}

/// Used for search input and comment viewing optional article body
pub fn top_header_style() -> Style {
    Style::new()
        .bg(Color::from_u32(0xb3ccff))
        .fg(Color::from_u32(0x00000))
}
