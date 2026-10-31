#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
pub use macos::*;

/// A change to the system theme
#[derive(Debug, Copy, Clone)]
pub enum Theme {
    /// Dark mode enabled
    Dark,
    /// Light mode enabled
    Light,
}

/// System setting change.
#[derive(Debug, Clone)]
pub enum SettingChange {
    /// System theme changed
    Theme(Theme),
    /// System font scale changed
    FontScale(f64),
    /// Default system font selection changed
    SystemUIFont(FontData),
    /// Default system monospace font selection changed
    SystemMonospaceFont(FontData),
}

/// System font data.
#[derive(Debug, Clone)]
pub struct FontData {
    /// Family name for the default UI font.
    pub family_name: String,
    /// Size if defined.
    pub size: Option<u8>,
}

/// System fonts.
pub struct SystemFonts {
    /// UI font.
    pub default_ui: Option<FontData>,
    /// Monspace font.
    pub monospace: Option<FontData>,
}
