use crate::AppState;
use gpui::{
    div, px, ParentElement, Render, Styled, Subscription, View, VisualContext, WindowContext,
};
use jiff::Zoned;

pub struct Footer {
    _state_subscription: Subscription,
    status_line: String,
}

impl Footer {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| Self {
            _state_subscription: cx.observe_global::<AppState>(move |footer: &mut Footer, cx| {
                println!("updating status line");
                footer.status_line = format!("Updated: {}", Zoned::now().strftime("%D %T"));
                cx.notify()
            }),
            status_line: String::from("Loading..."),
        })
    }
}

impl Render for Footer {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        div().h(px(50.)).child(self.status_line.clone())
    }
}
