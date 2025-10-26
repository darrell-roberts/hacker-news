//! Footer widget.
use std::time::Duration;

use crate::{App, app::Viewing};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Widget},
};

/// Footer widget displayed at the bottom.
pub struct FooterWidget<'a> {
    app: &'a App,
    style: Style,
}

impl<'a> FooterWidget<'a> {
    /// Create a new footer widget.
    pub fn new(app: &'a App) -> Self {
        Self {
            app,
            style: Style::default(),
        }
    }
}

impl FooterWidget<'_> {
    /// Set the style
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a> Widget for FooterWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        buf.set_style(area, self.style);
        match self.app.rebuild_progress.as_ref() {
            Some(progress) => {
                let gauge = Gauge::default()
                    .block(Block::new().borders(Borders::all()).title("Updating Index"))
                    .percent(progress.percent())
                    .style(self.style)
                    .gauge_style(self.style);
                gauge.render(area, buf);
            }
            None => {
                let [url, index_stats] =
                    Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

                match &self.app.viewing_state {
                    Some(Viewing::Search(state)) => {
                        Line::raw(format!("Found {}", state.total_comments))
                    }
                    _ => Line::raw(self.app.select_item_url().unwrap_or_default()),
                }
                .render(url, buf);

                let active_index = self.app.search_context.read().unwrap().active_category();

                if let Some(stats) = self
                    .app
                    .config
                    .index_config
                    .index_stats
                    .iter()
                    .find(|s| s.category == active_index)
                {
                    let [left, right] = Layout::horizontal([
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                    ])
                    .areas(index_stats);
                    Line::from_iter([Span::raw(format!(
                        "Index ({}) ({})",
                        match local_time(stats.built_on) {
                            Some(built_on) => Span::raw(built_on),
                            None => Span::raw(""),
                        },
                        duration_string(stats.build_time)
                    ))])
                    .render(left, buf);
                    Line::raw(format!("Total comments: {}", stats.total_comments))
                        .alignment(Alignment::Right)
                        .render(right, buf);
                }
            }
        }
    }
}

fn local_time(ts: u64) -> Option<String> {
    let tz = iana_time_zone::get_timezone().ok()?.parse::<Tz>().ok()?;
    let build_date =
        DateTime::<Utc>::from_timestamp(ts.try_into().ok()?, 0).map(|dt| dt.with_timezone(&tz))?;
    Some(build_date.format("%d/%m/%y %H:%M").to_string())
}

fn duration_string(elapsed: Duration) -> String {
    match (elapsed.as_secs(), elapsed.as_millis()) {
        (0, ms) => format!("{ms} ms"),
        (secs @ 1..60, ms) => format!("{secs} sec, {} ms", ms % 60),
        (secs @ 60..=3600, _) => {
            format!("{} minutes", secs / 60)
        }
        (secs, _) => {
            format!("{} hours", secs / 3600)
        }
    }
}
