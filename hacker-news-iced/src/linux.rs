//! Linux specific settings.
use crate::app::AppMsg;
use gio::{prelude::*, Settings};
use iced::{futures::Stream, Theme};
use log::{error, info};

/// Listen to dconf font scale changes.
pub fn listen_font_scale() -> impl Stream<Item = AppMsg> {
    use futures::channel::mpsc;
    // Create a futures channel for communication between threads
    let (tx, rx) = mpsc::unbounded::<AppMsg>();

    // Spawn a thread to handle gio Settings (since it's not Send)
    std::thread::spawn(move || {
        let scale_tx = tx.clone();
        let settings = Settings::new("org.gnome.desktop.interface");
        let _handler = settings.connect_changed(
            Some("text-scaling-factor"),
            move |settings, scale_factor| {
                let scale = settings.get::<f64>(scale_factor);
                info!("System font scale changed to: {scale}");

                // Use futures channel which works well between sync and async
                if let Err(err) = scale_tx.unbounded_send(AppMsg::SystemFontScale(scale)) {
                    error!("Failed to send font scale change: {err}");
                }
            },
        );

        let _handler =
            settings.connect_changed(Some("color-scheme"), move |settings, color_scheme| {
                let color_scheme = settings.get::<String>(color_scheme);
                let theme = theme(&color_scheme);

                if let Err(err) = tx.unbounded_send(AppMsg::ChangeTheme(theme)) {
                    error!("Failed to send theme change: {err}");
                }
            });

        // Keep the thread alive to maintain the gio connection
        let main_loop = glib::MainLoop::new(None, false);
        main_loop.run();
    });

    // Convert the receiver to a stream that sends AppMsg
    rx
}

/// Read the initial dconf font scale
pub fn initial_font_scale() -> f64 {
    let settings = Settings::new("org.gnome.desktop.interface");
    settings.get::<f64>("text-scaling-factor")
}

pub fn initial_theme() -> Theme {
    let color_schema = Settings::new("org.gnome.desktop.interface").get::<String>("color-scheme");
    theme(&color_schema)
}

fn theme(color_scheme: &str) -> Theme {
    match color_scheme {
        "default" | "prefer-light" => Theme::Light,
        "prefer-dark" => Theme::Dark,
        _ => Theme::Light,
    }
}
