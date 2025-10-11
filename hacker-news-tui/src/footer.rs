//! Footer widget.
use crate::App;
use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Constraint, Layout},
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
                let [url, index_stats] =
                    Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
                Line::raw(self.app.select_item_url().unwrap_or_default()).render(url, buf);
                if let Some(built) = self
                    .app
                    .index_stats
                    .and_then(|stats| local_time(stats.built_on))
                {
                    Line::raw(format!("Updated {built}")).render(index_stats, buf);
                }
            }
        }
    }
}

fn local_time(ts: u64) -> Option<String> {
    let build_date = DateTime::<Utc>::from_timestamp(ts.try_into().ok()?, 0)?;
    let local = build_date.naive_local();
    Some(local.format("%c").to_string())
}
