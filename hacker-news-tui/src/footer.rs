//! Footer widget.
use std::{borrow::Cow, time::Duration};

use crate::App;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{America::New_York, Tz};
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
                if let Some(stats) = self.app.index_stats {
                    Line::from_iter([
                        Cow::Owned(format!(
                            "Updated in {} on ",
                            duration_string(stats.build_time)
                        )),
                        match local_time(stats.built_on) {
                            Some(built_on) => Cow::Owned(built_on),
                            None => Cow::Borrowed(""),
                        },
                        Cow::Owned(format!(". Total comments: {}", stats.total_comments)),
                    ])
                    .render(index_stats, buf);
                }
            }
        }
    }
}

fn local_time(ts: u64) -> Option<String> {
    let tz: Tz = iana_time_zone::get_timezone().ok()?.parse().ok()?;
    let build_date =
        DateTime::<Utc>::from_timestamp(ts.try_into().ok()?, 0).map(|dt| dt.with_timezone(&tz))?;
    Some(build_date.format("%d/%m/%y %H:%M").to_string())
}

fn duration_string(elapsed: Duration) -> String {
    match (elapsed.as_secs(), elapsed.as_millis()) {
        (0, ms) => format!("{ms} ms"),
        (secs @ 1..60, ms) => format!("{secs} seconds and {} ms", ms % 60),
        (secs @ 60..=3600, _) => {
            format!("{} minutes", secs / 60)
        }
        (secs, _) => {
            format!("{} hours", secs / 3600)
        }
    }
}
