use gpui::{div, FontWeight, ParentElement, Render, Styled, View, VisualContext, WindowContext};
use hacker_news_api::Item;

pub struct CommentView {
    item: Item,
}

impl CommentView {
    pub fn new(cx: &mut WindowContext, item: Item) -> View<Self> {
        cx.new_view(|_| Self { item })
    }
}

impl Render for CommentView {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        div()
            .p_1()
            .child(render_rich_text(self.item.text.as_deref().unwrap_or("")))
    }
}

fn render_rich_text(escaped_text: &str) -> impl gpui::IntoElement {
    let elements = html_sanitizer::parse_elements(escaped_text);

    let child_iter = elements.into_iter().flat_map(|element| match element {
        html_sanitizer::Element::Text(text) => Some(div().child(text.to_string())),
        html_sanitizer::Element::Link(link) => link
            .attributes
            .iter()
            .find(|a| a.name == "href")
            .map(|attr| {
                let text = attr.value.as_str();
                if link.children.is_empty() {
                    div().child(text.to_string())
                } else {
                    div().child(link.children)
                }
            }),
        html_sanitizer::Element::Escaped(c) => Some(div().child(c.to_string())),
        html_sanitizer::Element::Paragraph => Some(div()),
        html_sanitizer::Element::Code(c) => Some(div().child(c)),
        html_sanitizer::Element::Italic(text) => Some(div().italic().child(text)),
        html_sanitizer::Element::Bold(text) => {
            Some(div().font_weight(FontWeight::BOLD).child(text))
        }
    });

    div().children(child_iter)
}
