//! Footer widget.
use crate::App;
use ratatui::{
    text::Line,
    widgets::{Block, Borders, Gauge, Widget},
};

/// Footer widget displayed at the bottom.
pub struct FooterWidget<'a> {
    app: &'a App,
}

impl<'a> FooterWidget<'a> {
    /// Create a new footer widget.
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for FooterWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match self.app.rebuild_progress.as_ref() {
            Some(progress) => {
                let gauge = Gauge::default()
                    .block(Block::new().borders(Borders::all()).title("Updating Index"))
                    .percent(progress.percent());
                gauge.render(area, buf);
            }
            None => {
                Line::raw(self.app.select_item_url().unwrap_or_default()).render(area, buf);
            }
        }
    }
}
