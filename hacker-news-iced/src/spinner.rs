use std::f32::consts::PI;

use iced::{
    advanced::{self, graphics::geometry::frame::Backend, layout, widget::tree, Widget},
    widget::canvas,
    Color, Element, Length, Radians, Renderer, Transformation, Vector,
};

pub struct Spinner {
    size: f32,
    bar_height: f32,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: 20.,
            bar_height: 2.,
        }
    }
}

const MIN_ANGLE: Radians = Radians(PI / 8.0);
const WRAP_ANGLE: Radians = Radians(2.0 * PI - PI / 4.0);

impl<M, T> Widget<M, T, Renderer> for Spinner {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size {
            width: Length::Fixed(self.size),
            height: Length::Fixed(self.size),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        layout::atomic(limits, self.size, self.size)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &T,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        use advanced::Renderer as _;

        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let track_radius = frame.width() / 2.0 - self.bar_height;
            let track_path = canvas::Path::circle(frame.center(), track_radius);

            // frame.stroke(
            //     &track_path,
            //     canvas::Stroke::default()
            //         .with_color(Color::from_rgb8(255, 0, 0))
            //         .with_width(self.bar_height),
            // );

            let mut builder = canvas::path::Builder::new();
            let start = Radians(10. * 2.0 * PI);

            builder
                .arc(canvas::path::Arc {
                    center: frame.center(),
                    radius: track_radius,
                    start_angle: start,
                    end_angle: start - Radians(1.0), //+ MIN_ANGLE + WRAP_ANGLE,
                })
                .rotate(3.);

            let bar_path = builder.build();

            frame.stroke(
                &bar_path,
                canvas::Stroke::default()
                    .with_color(Color::BLACK)
                    .with_width(self.bar_height),
            );
        });

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            use iced::advanced::graphics::geometry::Renderer as _;

            renderer.draw_geometry(geometry);
        });
    }
}

#[derive(Default)]
struct State {
    cache: canvas::Cache,
}

impl<'a, Message, Theme> From<Spinner> for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
{
    fn from(spinner: Spinner) -> Self {
        Self::new(spinner)
    }
}
