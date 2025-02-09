//! Common UI elements used by multiple views.
use crate::{app::AppMsg, footer::FooterMsg};
use iced::{
    alignment::Vertical,
    widget::{
        self,
        text::{IntoFragment, Shaping},
        tooltip::Position,
        Tooltip,
    },
    Background, Element, Length, Task, Theme,
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

/// Trait that creates a common pagination element.
pub trait PaginatingView<Message>
where
    Message: Clone,
{
    /// Create a pagination element that shows buttons for forward,
    /// back and page numbers.
    fn pagination_element<'a>(&'a self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let (div, rem) = (self.full_count() / 10, self.full_count() % 10);
        let max = if rem > 0 { div + 1 } else { div };
        let pages = (1..=max).map(|page| {
            widget::button(
                widget::container(widget::text(format!("{page}")))
                    .style(move |theme: &Theme| {
                        let palette = theme.extended_palette();
                        if page == self.current_page() {
                            widget::container::rounded_box(theme)
                                .background(palette.secondary.strong.color)
                        } else {
                            widget::container::transparent(theme)
                        }
                    })
                    .padding(5),
            )
            .style(widget::button::text)
            .padding(0)
            .on_press(self.jump_page(page))
            .into()
        });

        widget::container(
            widget::Row::new()
                .push(
                    widget::button(widget::text("←").shaping(Shaping::Advanced))
                        .on_press_maybe(self.current_page().gt(&1).then(|| self.go_back())),
                )
                .extend(pages)
                .push(
                    widget::button(widget::text("→").shaping(Shaping::Advanced)).on_press_maybe(
                        (self.current_page() < (self.full_count() / 10) + 1)
                            .then_some(self.go_forward()),
                    ),
                )
                .spacing(2)
                .align_y(Vertical::Center)
                .wrap(),
        )
        .center_x(Length::Fill)
        .padding([5, 0])
        .into()
    }

    /// Message to jump to page.
    fn jump_page(&self, page: usize) -> Message;

    /// Message to go back.
    fn go_back(&self) -> Message;

    /// Message to go forward.
    fn go_forward(&self) -> Message;

    /// Full count of pagination items.
    fn full_count(&self) -> usize;

    /// Current page.
    fn current_page(&self) -> usize;
}

/// Common error task to display errors in the footer.
pub fn error_task(err: impl ToString) -> Task<AppMsg> {
    Task::done(FooterMsg::Error(err.to_string())).map(AppMsg::Footer)
}
