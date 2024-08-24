//! Header view.
use gpui::{
    div, px, IntoElement, ParentElement, Render, Styled, View, VisualContext, WindowContext,
};

/// Header view
pub struct Header;

impl Header {
    /// Create a new header view.
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_| Self)
    }
}

impl Render for Header {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl IntoElement {
        let col1 = ["Top", "Best", "New"]
            .into_iter()
            .map(|label| div().child(label));

        div()
            .flex()
            .flex_row()
            .font_family("Roboto, sans-serif")
            .text_size(px(24.0))
            .gap_x(px(10.0))
            .w_full()
            .children(col1)
            .px_1()
    }
}
