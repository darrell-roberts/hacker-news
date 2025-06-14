//! Linux specific settings.
use crate::app::AppMsg;
use futures::channel::mpsc;
use gio::{prelude::*, Settings};
use iced::{futures::Stream, Theme};
use log::{error, info};

/// Listen to GSettings/dconf changes.
pub fn listen_to_system_changes() -> impl Stream<Item = AppMsg> {
    let (tx, rx) = mpsc::unbounded::<AppMsg>();

    std::thread::spawn(move || {
        let scale_tx = tx.clone();
        let settings = Settings::new("org.gnome.desktop.interface");

        let _handler = settings.connect_changed(
            Some("text-scaling-factor"),
            move |settings, scale_factor| {
                let scale = settings.get::<f64>(scale_factor);
                info!("System font scale changed to: {scale}");

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

    rx
}

/// Read the initial dconf font scale
pub fn initial_font_scale() -> f64 {
    Settings::new("org.gnome.desktop.interface").get::<f64>("text-scaling-factor")
}

pub fn initial_theme() -> Theme {
    let color_schema = Settings::new("org.gnome.desktop.interface").get::<String>("color-scheme");
    theme(&color_schema)
}

fn theme(color_scheme: &str) -> Theme {
    match color_scheme {
        "default" | "prefer-light" => Theme::Light,
        "prefer-dark" => Theme::SolarizedDark,
        _ => Theme::Light,
    }
}
