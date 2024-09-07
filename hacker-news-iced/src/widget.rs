use iced::{
    advanced::{self, mouse, renderer, widget, Widget},
    event,
    widget::container,
    Element, Event, Length, Padding, Point, Rectangle,
};

pub struct Hoverable<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_hover: Message,
    on_exit: Message,
    padding: Padding,
}

pub fn hoverable<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    on_hover: Message,
    on_exit: Message,
) -> Hoverable<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog,
{
    Hoverable::new(content.into(), on_hover, on_exit)
}

impl<'a, Message, Theme, Renderer> Hoverable<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog,
{
    pub fn new(
        content: Element<'a, Message, Theme, Renderer>,
        on_hover: Message,
        on_exit: Message,
    ) -> Self {
        Self {
            content,
            on_hover,
            on_exit,
            padding: Padding::ZERO,
        }
    }

    pub fn padding<P>(mut self, padding: P) -> Self
    where
        P: Into<Padding>,
    {
        self.padding = padding.into();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum State {
    #[default]
    Idle,
    Hovered {
        cursor_position: Point,
    },
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Hoverable<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: container::Catalog,
    Renderer: advanced::Renderer,
{
    fn tag(&self) -> advanced::widget::tree::Tag {
        advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> advanced::widget::tree::State {
        advanced::widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<advanced::widget::Tree> {
        vec![advanced::widget::tree::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> iced::Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> iced::Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &self,
        tree: &mut advanced::widget::Tree,
        renderer: &Renderer,
        limits: &advanced::layout::Limits,
    ) -> advanced::layout::Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: advanced::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        _event: Event,
        layout: advanced::Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();
        let was_idle = *state == State::Idle;

        *state = cursor
            .position_over(layout.bounds())
            .map(|cursor_position| State::Hovered { cursor_position })
            .unwrap_or_default();

        match (was_idle, matches!(state, State::Hovered { .. })) {
            (false, false) => shell.publish(self.on_exit.clone()),
            (false, true) => shell.publish(self.on_hover.clone()),
            _ => (),
        }

        // let is_idle = *state == State::Idle;

        // if was_idle != is_idle {
        //     shell.invalidate_layout();
        // }

        event::Status::Ignored
    }
}

impl<'a, Message, Theme, Renderer> From<Hoverable<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: container::Catalog + 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn from(value: Hoverable<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
