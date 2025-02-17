//! Common UI elements used by multiple views.
use crate::{app::AppMsg, footer::FooterMsg};
use hacker_news_search::{api::CommentStack, SearchContext};
use iced::{
    alignment::Vertical,
    font::{Style, Weight},
    mouse,
    widget::{
        self,
        canvas::{Frame, Geometry, Path, Stroke},
        text::{IntoFragment, Shaping},
        tooltip::Position,
        Tooltip,
    },
    Background, Color, Element, Font, Length, Point, Rectangle, Renderer, Task, Theme,
};
use std::sync::{Arc, RwLock};

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

/// Task to open comment with full parent thread.
pub fn show_thread(search_context: Arc<RwLock<SearchContext>>, comment_id: u64) -> Task<AppMsg> {
    let g = search_context.read().unwrap();
    match g.parents(comment_id) {
        Ok(CommentStack { comments, story }) => {
            let story_id = story.id;
            Task::done(AppMsg::OpenComment {
                parent_id: story_id,
                article: story,
                comment_stack: comments,
            })
        }
        Err(err) => error_task(err),
    }
}

/// Font extension trait.
pub trait FontExt {
    /// Bold font.
    fn bold(self) -> Self;
    /// Italic font.
    fn italic(self) -> Self;
    /// Light weight.
    fn weight_light(self) -> Self;
}

impl FontExt for Font {
    fn bold(self) -> Self {
        Self {
            weight: Weight::Bold,
            ..self
        }
    }

    fn italic(self) -> Self {
        Self {
            style: Style::Italic,
            ..self
        }
    }

    fn weight_light(self) -> Self {
        Self {
            weight: Weight::Light,
            ..self
        }
    }
}

/// An L-Shape connector.
pub struct LShape {
    vertical_height: f32,
    horizontal_length: f32,
}

impl LShape {
    /// New L-Shape connector.
    pub fn new(vertical_height: f32, horizontal_length: f32) -> Self {
        Self {
            vertical_height,
            horizontal_length,
        }
    }
}

impl<Message> widget::canvas::Program<Message> for LShape {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Create path for L shape
        let path = Path::new(|builder| {
            // Start at top
            builder.move_to(Point::new(10.0, 0.0));
            // Draw vertical line down
            builder.line_to(Point::new(10.0, self.vertical_height));
            // Draw horizontal line right
            builder.line_to(Point::new(
                10.0 + self.horizontal_length,
                self.vertical_height,
            ));
        });

        let dark = theme.extended_palette().is_dark;

        // Draw the path
        frame.stroke(
            &path,
            Stroke::default()
                .with_width(1.0)
                .with_line_join(widget::canvas::LineJoin::Round)
                .with_line_cap(widget::canvas::LineCap::Round)
                .with_color(if dark { Color::WHITE } else { Color::BLACK }),
        );

        vec![frame.into_geometry()]
    }
}
