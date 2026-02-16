//! A hacker news reader for the terminal.

use std::error::Error;

use crate::{app::App, config::load_config};
use color_eyre::eyre::Context;

use hacker_news_config::init_logger;
#[cfg(target_family = "unix")]
use hacker_news_config::limits::check_nofiles_limit;
use log::{debug, error};

mod app;
mod articles;
mod comments;
mod config;
mod events;
mod footer;
mod help;
mod search;
mod styles;

/// Starts ratatui and runs [`App`]. This runs
/// in tokio in order to use the `hacker-news-search`
/// API's which are async.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    init_logger("hacker-news-tui")?;

    #[cfg(target_family = "unix")]
    check_nofiles_limit();

    let config = load_config()
        .inspect_err(|err| {
            error!("No config file: {err}");
        })
        .unwrap_or_default();

    debug!("Config: {config:#?}");

    let terminal = ratatui::init();
    let result = App::new(config)?.run(terminal);
    ratatui::restore();
    Ok(result.context("Run failed")?)
}
