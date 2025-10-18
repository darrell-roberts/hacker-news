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
#[derive(Debug, Copy, Clone)]
pub enum SettingChange {
    /// System theme changed
    Theme(Theme),
    /// System font scale changed
    FontScale(f64),
}
