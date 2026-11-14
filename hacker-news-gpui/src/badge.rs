//! Standalone Badge Component (inspired by gpui-kit)
use gpui::{
    AnyElement, App, ClickEvent, IntoElement, ParentElement, Refineable as _, RenderOnce, Rgba,
    SharedString, StyleRefinement, Styled, Window, div, prelude::*, rems, rgb,
};

type BadgeClicked = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Badge {
    style: StyleRefinement,
    count: Option<SharedString>,
    color: Rgba,
    text_color: Rgba,
    border_color: Option<Rgba>,
    on_click: Option<BadgeClicked>,
    children: Vec<AnyElement>,
}

impl Badge {
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
            count: None,
            color: rgb(0xEF4444), // Vibrant notification red
            text_color: rgb(0xFFFFFF),
            border_color: None,
            on_click: None,
            children: Vec::new(),
        }
    }

    pub fn count(mut self, count: Option<SharedString>) -> Self {
        self.count = count;
        self
    }

    pub fn color(mut self, color: Rgba) -> Self {
        self.color = color;
        self
    }

    pub fn text_color(mut self, color: Rgba) -> Self {
        self.text_color = color;
        self
    }

    pub fn ring_color(mut self, color: Rgba) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn on_click(
        mut self,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }
}

impl Default for Badge {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Badge {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for Badge {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut container = div().relative();
        container.style().refine(&self.style);

        container
            .children(self.children)
            .when_some(self.count, |parent, count_label| {
                parent.child(
                    div()
                        .id("badge")
                        .absolute()
                        .top(rems(-0.35))
                        .right(rems(0.5))
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(rems(1.0))
                        .min_w(rems(1.0))
                        .px(rems(0.35))
                        .rounded_full()
                        .bg(self.color)
                        .text_color(self.text_color)
                        .text_size(rems(0.65))
                        .font_weight(gpui::FontWeight::BOLD)
                        .shadow_sm()
                        .when_some(self.border_color, |el, border| {
                            el.border_1().border_color(border)
                        })
                        .when_some(self.on_click, |el, cb| {
                            el.cursor_pointer()
                                .on_click(move |event, window, app| cb(event, window, app))
                        })
                        .child(count_label),
                )
            })
    }
}
