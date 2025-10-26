//! Re-usable styles
use ratatui::style::{Color, Style, Stylize as _};

/// An item is selected
pub fn selected_style() -> Style {
    Style::new()
        .fg(Color::from_u32(0xe6e600))
        .bg(Color::from_u32(0xcc0000))
        .bold()
}
