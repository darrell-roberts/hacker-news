//! Simple themes
use gpui::{rgb, Global, WindowAppearance};

mod light {
    pub const TEXT_COLOR: u32 = 0x444444;
    pub const BACKGROUND_COLOR: u32 = 0xb3b3b3;
    pub const HOVER: u32 = 0xe0e4eb;
    pub const TEXT_INCREASING: u32 = 0x15803D;
    pub const TEXT_DECREASING: u32 = 0xd70000;
    pub const SURFACE: u32 = 0xeeeeee;
    pub const BORDER: u32 = 0xb5bfc9;
    pub const COMMENT_BORDER: u32 = 0xff9900;
}

mod dark {
    pub const TEXT_COLOR: u32 = 0xbfbfbf;
    pub const BACKGROUND_COLOR: u32 = 0x121212;
    pub const HOVER: u32 = 0x2A2A2A;
    pub const TEXT_INCREASING: u32 = 0x57e389;
    pub const TEXT_DECREASING: u32 = 0xed333b;
    pub const SURFACE: u32 = 0x1e1e1e;
    pub const BORDER: u32 = 0x3a3a3a;
    pub const COMMENT_BORDER: u32 = 0xb36b00;
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

    pub fn hover(&self) -> gpui::Rgba {
        rgb(match self {
            Theme::Dark => dark::HOVER,
            Theme::Light => light::HOVER,
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

    pub fn surface(&self) -> gpui::Rgba {
        rgb(match self {
            Theme::Dark => dark::SURFACE,
            Theme::Light => light::SURFACE,
        })
    }

    pub fn border(&self) -> gpui::Rgba {
        rgb(match self {
            Theme::Dark => dark::BORDER,
            Theme::Light => light::BORDER,
        })
    }

    pub fn comment_border(&self) -> gpui::Rgba {
        rgb(match self {
            Theme::Dark => dark::COMMENT_BORDER,
            Theme::Light => light::COMMENT_BORDER,
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
