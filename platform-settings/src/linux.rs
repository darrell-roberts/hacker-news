//! Linux specific settings.
//!
//! Uses gsettings to get Gnome desktop interface settings.
use crate::{FontData, SettingChange, SystemFonts, Theme};
use futures::{Stream, channel::mpsc};
use gio::{Settings, prelude::*};
use log::{error, info, warn};

const GNOME_INTERFACE_SCHEMA_ID: &str = "org.gnome.desktop.interface";
const KEY_TEXT_SCALING_FACTOR: &str = "text-scaling-factor";
const KEY_COLOR_SCHEME: &str = "color-scheme";
const KEY_FONT_NAME: &str = "font-name";
const KEY_MONOSPACE_FONT_NAME: &str = "monospace-font-name";

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
                    let font_selection = parse_font(&settings.get::<String>(font_name));

                    match font_selection {
                        Some(font_selection) => {
                            if let Err(err) =
                                tx.unbounded_send(SettingChange::SystemUIFont(font_selection))
                            {
                                error!("Failed to send the system font change: {err}");
                            }
                        }
                        None => {
                            warn!("Failed to parse new font selection: {font_name}");
                        }
                    }
                }
            });

            let _handler = settings.connect_changed(Some(KEY_MONOSPACE_FONT_NAME), {
                let tx = tx.clone();
                move |settings, font_name| {
                    let font_selection = parse_font(&settings.get::<String>(font_name));

                    match font_selection {
                        Some(font_selection) => {
                            if let Err(err) = tx
                                .unbounded_send(SettingChange::SystemMonospaceFont(font_selection))
                            {
                                error!("Failed to send the system font change: {err}");
                            }
                        }
                        None => {
                            warn!("Failed to parse new font selection: {font_name}");
                        }
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
        default_ui: parse_font(&gsettings.get::<String>(KEY_FONT_NAME)),
        monospace: parse_font(&gsettings.get::<String>(KEY_MONOSPACE_FONT_NAME)),
    }
}

const STYLE_WORDS: &[&str] = &[
    "Thin",
    "Ultra-Light",
    "Extra-Light",
    "Light",
    "Semi-Light",
    "Book",
    "Regular",
    "Medium",
    "Semi-Bold",
    "Bold",
    "Ultra-Bold",
    "Extra-Bold",
    "Heavy",
    "Black",
    "Italic",
    "Oblique",
    "Roman",
    "Small-Caps",
    "Condensed",
    "Semi-Condensed",
    "Expanded",
    "Semi-Expanded",
];

fn parse_font(font_data: &str) -> Option<FontData> {
    let mut size = None;
    let mut words = font_data.split_whitespace().collect::<Vec<_>>();

    // Size is the last word.
    if words.len() > 1
        && let Ok(font_size) = words[words.len() - 1].parse::<u8>()
    {
        size = Some(font_size);
        words.pop();
    }

    // Strip out styles if any.
    while words.len() > 1 {
        let w = words[words.len() - 1];
        if STYLE_WORDS.iter().any(|k| k.eq_ignore_ascii_case(w)) {
            words.pop();
        } else {
            break;
        }
    }

    Some(FontData {
        family_name: words.join(" "),
        size,
    })
}

#[cfg(test)]
mod test {
    use super::parse_font;

    #[test]
    fn test_font_parse() {
        let test_font = "Noto Sans 12";

        let font_data = parse_font(test_font).expect("Parse failed");
        assert_eq!(font_data.family_name, "Noto Sans");
        assert_eq!(font_data.size, Some(12));

        let test_font = "Hack 12";
        let font_data = parse_font(test_font).expect("Parse failed");
        assert_eq!(font_data.family_name, "Hack");
        assert_eq!(font_data.size, Some(12));
    }
}
