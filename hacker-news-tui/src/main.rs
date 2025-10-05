//! A hacker news reader for the terminal.
use app_dirs2::get_app_dir;
use flexi_logger::{Age, Cleanup, Criterion, FileSpec, Naming};

use crate::app::{APP_INFO, App};

mod app;
mod articles;
mod comments;
mod events;
mod footer;

/// Starts ratatui and runs [`App`]. This runs
/// in tokio in order to use the `hacker-news-search`
/// API's which are async.
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let log_dir = get_app_dir(app_dirs2::AppDataType::UserData, &APP_INFO, "logs")?;

    println!("Writing logs to {log_dir:?}");

    let _logger = flexi_logger::Logger::try_with_env_or_str("info")?
        .log_to_file(
            FileSpec::default()
                .directory(log_dir)
                .basename("hacker-news"),
        )
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(5),
        )
        .print_message()
        .start()?;

    let terminal = ratatui::init();
    let result = App::new()?.run(terminal);
    ratatui::restore();
    result
}
