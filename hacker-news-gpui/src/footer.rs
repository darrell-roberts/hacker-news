use gpui::{div, px, Render, Styled, View, VisualContext, WindowContext};

pub struct Footer;

impl Footer {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|_cx| Self)
    }
}

impl Render for Footer {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        div().h(px(50.))
    }
}
