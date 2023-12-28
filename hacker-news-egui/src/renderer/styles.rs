//! Common re-usable styles for frames.
use egui::{epaint::Shadow, Color32, Frame, Margin, Rounding, Stroke};

pub fn central_panel_frame() -> Frame {
    Frame {
        fill: Color32::from_rgb(245, 243, 240),
        inner_margin: Margin {
            left: 5.,
            right: 5.,
            top: 5.,
            bottom: 5.,
        },
        ..Default::default()
    }
}

pub fn user_window_frame() -> Frame {
    window_frame(Color32::from_rgb(220, 245, 247))
}

pub fn comment_window_frame() -> Frame {
    window_frame(Color32::from_rgb(246, 247, 176))
}

pub fn article_text_window_frame() -> Frame {
    window_frame(Color32::from_rgb(195, 250, 248))
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
        .shadow(Shadow::small_light())
}

pub fn user_bubble_frame() -> Frame {
    bubble_frame(Color32::LIGHT_BLUE)
}

pub fn comment_bubble_frame() -> Frame {
    bubble_frame(Color32::LIGHT_YELLOW)
}

fn bubble_frame(fill_color: Color32) -> Frame {
    Frame::none()
        .fill(fill_color)
        .outer_margin(Margin {
            top: 5.,
            left: 10.,
            right: 10.,
            bottom: 5.,
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

pub fn article_text_bubble_frame() -> Frame {
    bubble_frame(Color32::from_rgb(224, 251, 255))
}
