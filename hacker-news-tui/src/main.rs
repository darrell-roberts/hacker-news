//! A hacker news reader for the terminal.

use std::error::Error;

use crate::{app::App, config::CONFIG_FILE};
use color_eyre::eyre::Context;
use hacker_news_config::{init_logger, load_config};
use hacker_news_search::IndexStats;

mod app;
mod articles;
mod comments;
mod config;
mod events;
mod footer;
mod search;

/// Starts ratatui and runs [`App`]. This runs
/// in tokio in order to use the `hacker-news-search`
/// API's which are async.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    init_logger()?;

    let config = load_config::<IndexStats>(CONFIG_FILE).ok();

    let terminal = ratatui::init();
    let result = App::new(config)?.run(terminal);
    ratatui::restore();
    Ok(result.context("Run failed")?)
}
