//! Help widget
//!
use crate::app::View;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Row, Table, Widget},
};

/// Displays the help popup.
pub struct HelpWidget {
    viewing: View,
}

impl HelpWidget {
    /// Create a new help widget.
    pub fn new(viewing: View) -> Self {
        Self { viewing }
    }
}

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Right)
            .title("Help");

        let table = Table::new(
            match self.viewing {
                View::Articles => article_help(),
                View::Search => search_help(),
                View::Comments => comment_help(),
            },
            [Constraint::Max(15), Constraint::Fill(1)],
        )
        .block(block)
        .header(Row::new(["Key", "Usage"]).bottom_margin(1))
        .column_spacing(1)
        .style(
            Style::new()
                .bg(Color::from_u32(0xb3ccff))
                .fg(Color::from_u32(0x00000)),
        );

        table.render(area, buf);
    }
}

fn article_help<'a>() -> Vec<Row<'a>> {
    vec![
        Row::new(["j", "down"]),
        Row::new(["k", "up"]),
        Row::new(["pgup/ctrl+u", "page up"]),
        Row::new(["pgdwn/ctrl+f", "page down"]),
        Row::new(["home", "Scroll to top"]),
        Row::new(["end", "Scroll to end"]),
        Row::new(["->", "Next category"]),
        Row::new(["<-", "Previous category"]),
        Row::new(["r", "Rebuild category index"]),
        Row::new(["u", "Update selected article"]),
        Row::new(["o", "open article url"]),
        Row::new(["c", "open comments"]),
        Row::new(["/", "open comment search"]),
        Row::new(["q/Esc", "close/quit"]),
    ]
}

fn search_help<'a>() -> Vec<Row<'a>> {
    vec![
        Row::new(["j", "down"]),
        Row::new(["k", "up"]),
        Row::new(["pgup/ctrl+u", "page up"]),
        Row::new(["pgdwn/ctrl+f", "page down"]),
        Row::new(["home", "Scroll to top"]),
        Row::new(["end", "Scroll to end"]),
        Row::new(["->", "Next page"]),
        Row::new(["<-", "Previous page"]),
        Row::new(["Tab", "Select next comment"]),
        Row::new(["Shift+Tab", "Select previous comment"]),
        Row::new(["t", "open comment in thread"]),
        Row::new(["o", "open article url"]),
        Row::new(["c", "open comments"]),
        Row::new(["/", "open comment search"]),
        Row::new(["q/Esc", "close/quit"]),
    ]
}

fn comment_help<'a>() -> Vec<Row<'a>> {
    vec![
        Row::new(["j", "down"]),
        Row::new(["k", "up"]),
        Row::new(["pgup/ctrl+u", "page up"]),
        Row::new(["pgdwn/ctrl+f", "page down"]),
        Row::new(["home", "Scroll to top"]),
        Row::new(["end", "Scroll to end"]),
        Row::new(["->", "Next page"]),
        Row::new(["<-", "Previous page"]),
        Row::new(["Tab", "Select next comment"]),
        Row::new(["Shift+Tab", "Select previous comment"]),
        Row::new(["o", "open article url"]),
        Row::new(["c", "open comments"]),
        Row::new(["/", "open comment search"]),
        Row::new(["q/Esc", "close/quit"]),
    ]
}
