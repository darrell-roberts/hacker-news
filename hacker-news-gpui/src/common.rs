//! Common functions.
//!

use gpui::Image;
use std::sync::{Arc, LazyLock};

/// An embedded SVG comment image.
pub static COMMENT_IMAGE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        include_bytes!("../assets/comment.svg").into(),
    ))
});
