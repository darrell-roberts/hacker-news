use crate::App;
use ratatui::{
    text::Line,
    widgets::{Block, Borders, Gauge, Widget},
};

pub struct FooterView<'a> {
    app: &'a App,
}

impl<'a> FooterView<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for FooterView<'a> {
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
