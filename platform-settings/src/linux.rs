//! Linux specific settings.
//!
//! Uses gsettings to get Gnome desktop interface settings.
use crate::{SettingChange, Theme};
use futures::{Stream, channel::mpsc};
use gio::{Settings, prelude::*};
use log::{error, info};

const GNOME_INTERFACE_SCHEMA_ID: &str = "org.gnome.desktop.interface";
const KEY_TEXT_SCALING_FACTOR: &str = "text-scaling-factor";
const KEY_COLOR_SCHEME: &str = "color-scheme";
const KEY_FONT_NAME: &str = "font-name";
const KEY_MONOSPACE_FONT_NAME: &str = "monospace-font-name";

/// System fonts.
pub struct SystemFonts {
    /// UI font.
    pub default_ui: String,
    /// Monspace font.
    pub monospace: String,
}

/// Listen to GSettings changes.
pub fn listen_to_system_changes() -> impl Stream<Item = SettingChange> {
    let (tx, rx) = mpsc::unbounded::<SettingChange>();

    std::thread::Builder::new()
        .name("Background gsettings listener".into())
        .spawn(move || {
            let settings = Settings::new(GNOME_INTERFACE_SCHEMA_ID);

            let _handler = settings.connect_changed(Some(KEY_TEXT_SCALING_FACTOR), {
                let tx = tx.clone();
                move |settings, scale_factor| {
                    let scale = settings.get::<f64>(scale_factor);
                    info!("System font scale changed to: {scale}");

                    if let Err(err) = tx.unbounded_send(SettingChange::FontScale(scale)) {
                        error!("Failed to send font scale change: {err}");
                    }
                }
            });

            let _handler = settings.connect_changed(Some(KEY_COLOR_SCHEME), {
                let tx = tx.clone();
                move |settings, color_scheme| {
                    let color_scheme = settings.get::<String>(color_scheme);
                    let theme = theme(&color_scheme);

                    if let Err(err) = tx.unbounded_send(SettingChange::Theme(theme)) {
                        error!("Failed to send theme change: {err}");
                    }
                }
            });

            let _handler = settings.connect_changed(Some(KEY_FONT_NAME), {
                let tx = tx.clone();
                move |settings, font_name| {
                    let font_selection = settings.get::<String>(font_name);

                    if let Err(err) = tx.unbounded_send(SettingChange::FontName(font_selection)) {
                        error!("Failed to send the system font change: {err}");
                    }
                }
            });

            // Keep the thread alive to maintain the gio connection
            let main_loop = glib::MainLoop::new(None, false);
            main_loop.run();
        })
        .expect("Could not start background gsettings listener");

    rx
}

/// Read the gsettings for font scale `text-scaling-factor`.
/// G_VARIANT_TYPE_DOUBLE
/// <https://docs.gtk.org/glib/struct.VariantType.html>
pub fn initial_font_scale() -> f64 {
    Settings::new(GNOME_INTERFACE_SCHEMA_ID).get::<f64>(KEY_TEXT_SCALING_FACTOR)
}

/// Read the initial gsettings for `color-scheme`.
/// G_VARIANT_TYPE_VARIANT
/// <https://docs.gtk.org/glib/struct.VariantType.html>
/// String variants "default", "prefer-dark", "prefer-light".
pub fn initial_theme() -> Theme {
    let color_schema = Settings::new(GNOME_INTERFACE_SCHEMA_ID).get::<String>(KEY_COLOR_SCHEME);
    theme(&color_schema)
}

fn theme(color_scheme: &str) -> Theme {
    match color_scheme {
        "default" | "prefer-light" => Theme::Light,
        "prefer-dark" => Theme::Dark,
        _ => Theme::Light,
    }
}

/// Get user system font settings from gsettings.
pub fn system_fonts() -> SystemFonts {
    let gsettings = Settings::new(GNOME_INTERFACE_SCHEMA_ID);

    SystemFonts {
        default_ui: gsettings.get(KEY_FONT_NAME),
        monospace: gsettings.get(KEY_MONOSPACE_FONT_NAME),
    }
}
