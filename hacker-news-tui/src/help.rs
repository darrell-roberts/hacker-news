//! Help widget
//!

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Cell, Row, Table, Widget},
};

use crate::app::View;

pub struct HelpWidget {
    viewing: View,
}

impl HelpWidget {
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
        Row::new([Cell::from("j"), Cell::from("down")]),
        Row::new([Cell::from("k"), Cell::from("up")]),
        Row::new([Cell::from("pgup/ctrl+u"), Cell::from("page up")]),
        Row::new([Cell::from("pgdwn/ctrl+f"), Cell::from("page down")]),
        Row::new([Cell::from("home"), Cell::from("Scroll to top")]),
        Row::new([Cell::from("end"), Cell::from("Scroll to end")]),
        Row::new([Cell::from("->"), Cell::from("Next category")]),
        Row::new([Cell::from("<-"), Cell::from("Previous category")]),
        Row::new([Cell::from("r"), Cell::from("Rebuild category index")]),
        Row::new([Cell::from("u"), Cell::from("Update selected article")]),
        Row::new([Cell::from("o"), Cell::from("open article url")]),
        Row::new([Cell::from("c"), Cell::from("open comments")]),
        Row::new([Cell::from("/"), Cell::from("open comment search")]),
        Row::new([Cell::from("q"), Cell::from("close/quit")]),
    ]
}

fn search_help<'a>() -> Vec<Row<'a>> {
    vec![
        Row::new([Cell::from("j"), Cell::from("down")]),
        Row::new([Cell::from("k"), Cell::from("up")]),
        Row::new([Cell::from("pgup/ctrl+u"), Cell::from("page up")]),
        Row::new([Cell::from("pgdwn/ctrl+f"), Cell::from("page down")]),
        Row::new([Cell::from("home"), Cell::from("Scroll to top")]),
        Row::new([Cell::from("end"), Cell::from("Scroll to end")]),
        Row::new([Cell::from("->"), Cell::from("Next page")]),
        Row::new([Cell::from("<-"), Cell::from("Previous page")]),
        Row::new([Cell::from("Tab"), Cell::from("Select next comment")]),
        Row::new([
            Cell::from("Shift+Tab"),
            Cell::from("Select previous comment"),
        ]),
        Row::new([Cell::from("t"), Cell::from("open comment in thread")]),
        Row::new([Cell::from("o"), Cell::from("open article url")]),
        Row::new([Cell::from("c"), Cell::from("open comments")]),
        Row::new([Cell::from("/"), Cell::from("open comment search")]),
        Row::new([Cell::from("q"), Cell::from("close/quit")]),
    ]
}

fn comment_help<'a>() -> Vec<Row<'a>> {
    vec![
        Row::new([Cell::from("j"), Cell::from("down")]),
        Row::new([Cell::from("k"), Cell::from("up")]),
        Row::new([Cell::from("pgup/ctrl+u"), Cell::from("page up")]),
        Row::new([Cell::from("pgdwn/ctrl+f"), Cell::from("page down")]),
        Row::new([Cell::from("home"), Cell::from("Scroll to top")]),
        Row::new([Cell::from("end"), Cell::from("Scroll to end")]),
        Row::new([Cell::from("->"), Cell::from("Next page")]),
        Row::new([Cell::from("<-"), Cell::from("Previous page")]),
        Row::new([Cell::from("Tab"), Cell::from("Select next comment")]),
        Row::new([
            Cell::from("Shift+Tab"),
            Cell::from("Select previous comment"),
        ]),
        Row::new([Cell::from("o"), Cell::from("open article url")]),
        Row::new([Cell::from("c"), Cell::from("open comments")]),
        Row::new([Cell::from("/"), Cell::from("open comment search")]),
        Row::new([Cell::from("q"), Cell::from("close/quit")]),
    ]
}
