//! A hacker news reader for the terminal.
use crate::app::App;

mod app;
mod articles;
mod events;
mod footer;

/// Starts ratatui and runs [`App`]. This runs
/// in tokio in order to use the `hacker-news-search`
/// API's which are async.
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new()?.run(terminal);
    ratatui::restore();
    result
}
