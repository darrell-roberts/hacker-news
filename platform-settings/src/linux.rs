//! Linux specific settings.
//!
//! Uses gsettings to get Gnome desktop interface settings.
use crate::{SettingChange, Theme};
use futures::{Stream, channel::mpsc};
use gio::{Settings, prelude::*};
use log::{error, info};

const GNOME_INTERFACE_SCHEMA_ID: &str = "org.gnome.desktop.interface";

/// Listen to GSettings/dconf changes.
pub fn listen_to_system_changes() -> impl Stream<Item = SettingChange> {
    let (tx, rx) = mpsc::unbounded::<SettingChange>();

    std::thread::Builder::new()
        .name("Background gsettings listener".into())
        .spawn(move || {
            let scale_tx = tx.clone();
            let settings = Settings::new(GNOME_INTERFACE_SCHEMA_ID);

            let _handler = settings.connect_changed(
                Some("text-scaling-factor"),
                move |settings, scale_factor| {
                    let scale = settings.get::<f64>(scale_factor);
                    info!("System font scale changed to: {scale}");

                    if let Err(err) = scale_tx.unbounded_send(SettingChange::FontScale(scale)) {
                        error!("Failed to send font scale change: {err}");
                    }
                },
            );

            let _handler =
                settings.connect_changed(Some("color-scheme"), move |settings, color_scheme| {
                    let color_scheme = settings.get::<String>(color_scheme);
                    let theme = theme(&color_scheme);

                    if let Err(err) = tx.unbounded_send(SettingChange::Theme(theme)) {
                        error!("Failed to send theme change: {err}");
                    }
                });

            // Keep the thread alive to maintain the gio connection
            let main_loop = glib::MainLoop::new(None, false);
            main_loop.run();
        })
        .expect("Could not start background gsettings listener");

    rx
}

/// Read the initial dconf font scale
pub fn initial_font_scale() -> f64 {
    Settings::new(GNOME_INTERFACE_SCHEMA_ID).get::<f64>("text-scaling-factor")
}

pub fn initial_theme() -> Theme {
    let color_schema = Settings::new(GNOME_INTERFACE_SCHEMA_ID).get::<String>("color-scheme");
    theme(&color_schema)
}

fn theme(color_scheme: &str) -> Theme {
    match color_scheme {
        "default" | "prefer-light" => Theme::Light,
        "prefer-dark" => Theme::Dark,
        _ => Theme::Light,
    }
}

/// Get the system default font.
pub fn initial_font_name() -> String {
    Settings::new(GNOME_INTERFACE_SCHEMA_ID).get::<String>("font-name")
}
