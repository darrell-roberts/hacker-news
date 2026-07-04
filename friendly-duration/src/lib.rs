use chrono::{DateTime, Utc};
use std::time::Duration;

/// Extract the duration from a UNIX time and convert duration into a human
/// friendly sentence.
pub fn parse_friendly_age(time: u64) -> Option<String> {
    let duration =
        DateTime::<Utc>::from_timestamp(time.try_into().ok()?, 0).map(|then| Utc::now() - then)?;

    let hours = duration.num_hours();
    let minutes = duration.num_minutes();
    let days = duration.num_days();

    match (days, hours, minutes) {
        (0, 0, 1) => "1 minute ago".to_string(),
        (0, 0, m) => format!("{m} minutes ago"),
        (0, 1, _) => "1 hour ago".to_string(),
        (0, h, _) => format!("{h} hours ago"),
        (1, _, _) => "1 day ago".to_string(),
        (d, _, _) => format!("{d} days ago"),
    }
    .into()
}

pub fn friendly_duration(duration: Duration) -> String {
    (duration.as_secs() / 60 != 0)
        .then(|| format!("{} min", duration.as_secs() / 60))
        .or_else(|| (duration.as_micros() < 1000).then(|| format!("{} ms", duration.as_millis())))
        .or_else(|| (duration.as_secs() >= 1).then(|| format!("{} secs", duration.as_secs() % 60)))
        .unwrap_or_default()
}
