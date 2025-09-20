//! Simple themes
use gpui::{rgb, Global, WindowAppearance};

mod light {
    pub const TEXT_COLOR: u32 = 0x424242;
    pub const BACKGROUND_COLOR: u32 = 0xeeeeee;
    pub const TEXT_LIGHT_BAR: u32 = 0xd1dbe0;
    pub const TEXT_INCREASING: u32 = 0x38841d;
    pub const TEXT_DECREASING: u32 = 0xc8321c;
    pub const STATUS_BAR_BACKGROUND: u32 = 0xdeddda;
}

mod dark {
    pub const TEXT_COLOR: u32 = 0xf6f5f4;
    pub const BACKGROUND_COLOR: u32 = 0x000000;
    pub const TEXT_LIGHT_BAR: u32 = 0x77767b;
    pub const TEXT_INCREASING: u32 = 0x57e389;
    pub const TEXT_DECREASING: u32 = 0xed333b;
    pub const STATUS_BAR_BACKGROUND: u32 = 0xc0bfbc;
}

#[derive(Debug, Copy, Clone)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn bg(&self) -> gpui::Rgba {
        rgb(match self {
            Theme::Dark => dark::BACKGROUND_COLOR,
            Theme::Light => light::BACKGROUND_COLOR,
        })
    }

    pub fn text_color(&self) -> gpui::Rgba {
        rgb(match self {
            Theme::Dark => dark::TEXT_COLOR,
            Theme::Light => light::TEXT_COLOR,
        })
    }

    pub fn text_light_bar(&self) -> gpui::Rgba {
        rgb(match self {
            Theme::Dark => dark::TEXT_LIGHT_BAR,
            Theme::Light => light::TEXT_LIGHT_BAR,
        })
    }

    pub fn text_increasing(&self) -> gpui::Rgba {
        rgb(match self {
            Theme::Dark => dark::TEXT_INCREASING,
            Theme::Light => light::TEXT_INCREASING,
        })
    }

    pub fn text_decreasing(&self) -> gpui::Rgba {
        rgb(match self {
            Theme::Dark => dark::TEXT_DECREASING,
            Theme::Light => light::TEXT_DECREASING,
        })
    }

    pub fn status_bar_background(&self) -> gpui::Rgba {
        rgb(match self {
            Theme::Dark => dark::STATUS_BAR_BACKGROUND,
            Theme::Light => light::STATUS_BAR_BACKGROUND,
        })
    }
}

impl Global for Theme {}

impl From<platform_settings::Theme> for Theme {
    fn from(theme: platform_settings::Theme) -> Self {
        match theme {
            platform_settings::Theme::Dark => Self::Dark,
            platform_settings::Theme::Light => Self::Light,
        }
    }
}

impl From<WindowAppearance> for Theme {
    fn from(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Light => Self::Light,
            WindowAppearance::VibrantLight => Self::Light,
            WindowAppearance::Dark => Self::Dark,
            WindowAppearance::VibrantDark => Self::Dark,
        }
    }
}
