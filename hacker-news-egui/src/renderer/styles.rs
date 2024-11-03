//! Common re-usable styles for frames.
use egui::{epaint::Shadow, Color32, Frame, Margin, Rounding, Stroke, Theme};

pub fn central_panel_frame(theme: &Theme) -> Frame {
    Frame {
        fill: match theme {
            Theme::Dark => Color32::from_rgb(33, 37, 41),
            Theme::Light => Color32::from_rgb(245, 243, 240),
            // Theme::Light => Color32::BLACK,
        },
        inner_margin: Margin {
            left: 0.,
            right: 0.,
            top: 5.,
            bottom: 5.,
        },
        ..Default::default()
    }
}

pub fn user_window_frame(theme: &Theme) -> Frame {
    window_frame(match theme {
        Theme::Dark => Color32::from_rgb(220, 245, 247),
        Theme::Light => Color32::from_rgb(220, 245, 247),
    })
}

pub fn comment_window_frame(theme: &Theme) -> Frame {
    let fill_color = match theme {
        Theme::Dark => Color32::from_hex("#344955").unwrap(),
        Theme::Light => Color32::from_rgb(252, 246, 228),
    };
    Frame::none()
        .fill(fill_color)
        .inner_margin(Margin {
            left: 5.,
            right: 5.,
            top: 5.,
            bottom: 5.,
        })
        .rounding(Rounding {
            nw: 8.,
            ne: 8.,
            sw: 8.,
            se: 8.,
        })
        .stroke(Stroke {
            color: Color32::BLACK,
            width: 1.,
        })
        .outer_margin(Margin {
            right: 5.,
            left: 5.,
            ..Default::default()
        })
}

pub fn article_text_window_frame(theme: &Theme) -> Frame {
    window_frame(match theme {
        Theme::Dark => Color32::from_rgb(195, 250, 248),
        Theme::Light => Color32::from_rgb(195, 250, 248),
    })
}

fn window_frame(fill_color: Color32) -> Frame {
    Frame::none()
        .fill(fill_color)
        .inner_margin(Margin {
            left: 5.,
            right: 5.,
            top: 5.,
            bottom: 5.,
        })
        .rounding(Rounding {
            nw: 8.,
            ne: 8.,
            sw: 8.,
            se: 8.,
        })
        .stroke(Stroke {
            color: Color32::BLACK,
            width: 1.,
        })
        .shadow(Shadow {
            offset: [1.0, 2.0].into(),
            ..Default::default()
        })
}

pub fn user_bubble_frame(theme: &Theme) -> Frame {
    bubble_frame(match theme {
        Theme::Dark => Color32::LIGHT_BLUE,
        Theme::Light => Color32::LIGHT_BLUE,
    })
}

pub fn comment_bubble_frame(theme: &Theme) -> Frame {
    bubble_frame(comment_bubble_color(theme))
}

pub fn comment_bubble_text(theme: &Theme) -> Color32 {
    match theme {
        egui::Theme::Dark => Color32::WHITE,
        egui::Theme::Light => Color32::BLACK,
    }
}

pub fn comment_bubble_color(theme: &Theme) -> Color32 {
    match theme {
        Theme::Dark => Color32::from_hex("#50727B").unwrap(),
        Theme::Light => Color32::LIGHT_YELLOW,
    }
}

fn bubble_frame(fill_color: Color32) -> Frame {
    Frame::none()
        .fill(fill_color)
        .outer_margin(Margin {
            top: 2.,
            left: 10.,
            right: 10.,
            bottom: 2.,
        })
        .inner_margin(Margin {
            top: 10.,
            left: 10.,
            right: 10.,
            bottom: 10.,
        })
        .rounding(Rounding {
            nw: 8.,
            ne: 8.,
            sw: 8.,
            se: 8.,
        })
        .stroke(Stroke {
            color: Color32::GRAY,
            width: 1.0,
        })
}

pub fn article_text_bubble_frame(theme: &Theme) -> Frame {
    bubble_frame(match theme {
        Theme::Dark => Color32::from_rgb(224, 251, 255),
        Theme::Light => Color32::from_rgb(224, 251, 255),
    })
}
