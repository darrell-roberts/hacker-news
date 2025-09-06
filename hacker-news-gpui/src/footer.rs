use crate::AppState;
use gpui::{
    div, px, App, AppContext as _, Entity, ParentElement, Render, Styled, Subscription, Window,
};
use jiff::Zoned;

pub struct Footer {
    _state_subscription: Subscription,
    status_line: String,
}

impl Footer {
    pub fn new(_cx: &mut Window, app: &mut App) -> Entity<Self> {
        app.new(|cx| Self {
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
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().h(px(50.)).child(self.status_line.clone())
    }
}
