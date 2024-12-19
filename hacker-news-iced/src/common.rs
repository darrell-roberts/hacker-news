use iced::{
    widget::{self, text::IntoFragment, tooltip::Position, Tooltip},
    Background, Element,
};

/// Create a tooltip with a common hover tooltip message style.
pub fn tooltip<'a, Message>(
    content: impl Into<Element<'a, Message>>,
    hover_msg: impl IntoFragment<'a>,
    position: Position,
) -> Tooltip<'a, Message>
where
    Message: 'a,
{
    widget::tooltip(
        content,
        widget::container(widget::text(hover_msg).color(iced::Color::WHITE))
            .style(|_| {
                widget::container::Style::default()
                    .background(Background::Color(iced::Color::BLACK))
                    .border(iced::border::rounded(8))
            })
            .padding(4),
        position,
    )
}
