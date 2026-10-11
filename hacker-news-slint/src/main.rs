// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use log::info;
use slint::CloseRequestResponse;

slint::include_modules!();

mod background;

fn main() -> Result<(), Box<dyn Error>> {
    let app_window = AppWindow::new()?;
    let app_window_weak = app_window.as_weak();

    hacker_news_config::init_logger("hacker-news-slint")?;

    background::subscribe_to_stories(app_window_weak);

    app_window.window().on_close_requested(|| {
        info!("Closing main window");
        CloseRequestResponse::HideWindow
    });

    app_window.run()?;

    Ok(())
}
