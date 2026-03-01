//! Custom scrollbar component for GPUI scroll containers.
//!
//! Renders a vertical scrollbar track with a draggable thumb that
//! reflects and controls the scroll position of a [`ScrollHandle`].
use crate::theme::Theme;
use gpui::{
    App, Div, Entity, InteractiveElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ParentElement, Pixels, ScrollHandle, SharedString, Styled, Window, div,
    prelude::*, px, relative,
};

const SCROLLBAR_WIDTH: Pixels = px(8.0);
const MIN_THUMB_HEIGHT: Pixels = px(20.0);
const TRACK_BORDER_RADIUS: Pixels = px(4.0);

/// State tracked for scrollbar drag interactions.
#[derive(Debug, Clone, Copy)]
struct DragState {
    /// The y-offset of the mouse within the thumb when the drag started.
    thumb_offset_y: Pixels,
}

/// A vertical scrollbar that pairs with a [`ScrollHandle`].
///
/// The scrollbar computes its thumb size and position from the handle's
/// `offset()`, `max_offset()`, and `bounds()`. Dragging the thumb or
/// clicking on the track updates the scroll position via `set_offset()`.
pub struct Scrollbar {
    scroll_handle: ScrollHandle,
    drag_state: Option<DragState>,
    id: SharedString,
}

impl Scrollbar {
    /// Create a new Scrollbar.
    pub fn new(
        app: &mut App,
        id: impl Into<SharedString>,
        scroll_handle: ScrollHandle,
    ) -> Entity<Self> {
        app.new(|_| Self {
            id: id.into(),
            scroll_handle,
            drag_state: None,
        })
    }

    /// Returns `true` when the user is actively dragging the scrollbar thumb.
    pub fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }

    /// Compute the fraction of content visible in the viewport (0.0–1.0).
    fn visible_fraction(&self) -> f32 {
        let max = self.scroll_handle.max_offset();
        let viewport_height = self.scroll_handle.bounds().size.height;
        let content_height = viewport_height + max.height.abs();
        if content_height <= px(0.) {
            return 1.0;
        }
        (viewport_height / content_height).clamp(0.0, 1.0)
    }

    /// Compute the scroll fraction (0.0 = top, 1.0 = fully scrolled).
    fn scroll_fraction(&self) -> f32 {
        let max = self.scroll_handle.max_offset();
        if max.height.abs() <= px(0.) {
            return 0.0;
        }
        let offset = self.scroll_handle.offset();
        // offset.y is negative as you scroll down; positive means overscroll past top
        if offset.y >= px(0.) {
            return 0.0;
        }
        (offset.y.abs() / max.height.abs()).clamp(0.0, 1.0)
    }

    /// Whether there is enough content to scroll (and thus show the scrollbar).
    fn is_scrollable(&self) -> bool {
        self.scroll_handle.max_offset().height.abs() > px(1.)
    }

    /// Set the scroll position from a fraction (0.0 = top, 1.0 = bottom).
    fn set_scroll_fraction(&self, fraction: f32) {
        let fraction = fraction.clamp(0.0, 1.0);
        let max = self.scroll_handle.max_offset();
        let mut offset = self.scroll_handle.offset();
        // offset.y should be negative (scrolling down)
        offset.y = -(max.height.abs() * fraction);
        self.scroll_handle.set_offset(offset);
    }

    /// Handle a mouse-move event during an active drag.
    ///
    /// This is intended to be called from a parent element's `on_mouse_move`
    /// handler so that dragging continues even when the mouse leaves the
    /// narrow scrollbar track.
    pub fn handle_drag_move(&mut self, event: &MouseMoveEvent, cx: &mut gpui::Context<Self>) {
        let Some(drag) = self.drag_state else {
            return;
        };
        if !event
            .pressed_button
            .is_some_and(|mouse_button| mouse_button == MouseButton::Left)
        {
            self.drag_state = None;
            cx.notify();
            return;
        }
        let bounds = self.scroll_handle.bounds();
        let track_height = bounds.size.height;
        if track_height <= px(0.0) {
            return;
        }
        let thumb_height = (track_height * self.visible_fraction()).max(MIN_THUMB_HEIGHT);
        let mouse_y = event.position.y - bounds.origin.y;
        let fraction =
            ((mouse_y - drag.thumb_offset_y) / (track_height - thumb_height)).clamp(0.0, 1.0);
        self.set_scroll_fraction(fraction);
        cx.notify();
    }

    /// Handle a mouse-up event to end an active drag.
    ///
    /// This is intended to be called from a parent element's `on_mouse_up`
    /// handler so that the drag ends reliably even when the mouse is
    /// outside the scrollbar track.
    pub fn handle_drag_end(&mut self, cx: &mut gpui::Context<Self>) {
        if self.drag_state.is_some() {
            self.drag_state = None;
            cx.notify();
        }
    }
}

impl Render for Scrollbar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme: Theme = window.appearance().into();

        let scrollbar_entity = cx.entity();

        let entity_for_track = scrollbar_entity.clone();
        let entity_for_move = scrollbar_entity.clone();
        let entity_for_up = scrollbar_entity.clone();
        let entity_for_down = scrollbar_entity.clone();

        div()
            .id(self.id.clone())
            .flex_shrink_0()
            .w(SCROLLBAR_WIDTH)
            .h_full()
            .bg(theme.surface())
            .rounded(TRACK_BORDER_RADIUS)
            .when(!self.is_scrollable(), |div| div.invisible())
            .child(
                div()
                    .id("scrollbar-track")
                    .w_full()
                    .h_full()
                    .relative()
                    // Click on track to jump to position
                    .on_mouse_down(
                        MouseButton::Left,
                        move |event: &MouseDownEvent, _window: &mut Window, app: &mut App| {
                            entity_for_track.update(app, |scrollbar, cx| {
                                let bounds = scrollbar.scroll_handle.bounds();
                                let track_height = bounds.size.height;
                                if track_height <= px(0.0) {
                                    return;
                                }
                                let thumb_height = (track_height * scrollbar.visible_fraction())
                                    .max(MIN_THUMB_HEIGHT);
                                // Center the thumb on the click position
                                let click_y = event.position.y - bounds.origin.y;
                                let fraction = ((click_y - thumb_height / 2.0)
                                    / (track_height - thumb_height))
                                    .clamp(0.0, 1.0);
                                scrollbar.set_scroll_fraction(fraction);
                                scrollbar.drag_state = Some(DragState {
                                    thumb_offset_y: thumb_height / 2.0,
                                });
                                cx.notify();
                            });
                        },
                    )
                    .on_mouse_move(
                        move |event: &MouseMoveEvent, _window: &mut Window, app: &mut App| {
                            entity_for_move.update(app, |scrollbar, cx| {
                                scrollbar.handle_drag_move(event, cx);
                            });
                        },
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        move |_event: &MouseUpEvent, _window: &mut Window, app: &mut App| {
                            entity_for_up.update(app, |scrollbar, cx| {
                                scrollbar.handle_drag_end(cx);
                            });
                        },
                    )
                    .child(thumb_element(
                        self.visible_fraction(),
                        self.scroll_fraction(),
                        theme,
                        entity_for_down,
                    )),
            )
    }
}

/// Builds the thumb div, positioned absolutely within the track.
fn thumb_element(
    visible_fraction: f32,
    scroll_fraction: f32,
    theme: Theme,
    entity: Entity<Scrollbar>,
) -> gpui::Stateful<Div> {
    // We use percentage based positioning via top/height style.
    // The thumb height = visible_fraction * 100%, min-clamped to MIN_THUMB_HEIGHT.
    // The thumb top = scroll_fraction * (track_height - thumb_height).
    //
    // Since we can't know the exact track pixel height at render time for percentage math,
    // we express thumb_height as a fraction and compute top offset accordingly.
    // For the min height clamp we use a fixed pixel value.
    let thumb_height_fraction = visible_fraction.clamp(0.05, 1.0);
    let available_fraction = 1.0 - thumb_height_fraction;
    let thumb_top_fraction = scroll_fraction * available_fraction;

    div()
        .id("scrollbar-thumb")
        .absolute()
        .w_full()
        .rounded(TRACK_BORDER_RADIUS)
        .bg(theme.border())
        .hover(|style| style.bg(theme.comment_border()))
        .cursor_pointer()
        .top(relative(thumb_top_fraction))
        .h(relative(thumb_height_fraction))
        .min_h(MIN_THUMB_HEIGHT)
        .on_mouse_down(
            MouseButton::Left,
            move |event: &MouseDownEvent, _window: &mut Window, app: &mut App| {
                entity.update(app, |scrollbar, cx| {
                    let bounds = scrollbar.scroll_handle.bounds();
                    let track_height = bounds.size.height;
                    let thumb_height =
                        (track_height * scrollbar.visible_fraction()).max(MIN_THUMB_HEIGHT);
                    let thumb_top = scrollbar.scroll_fraction() * (track_height - thumb_height);
                    let mouse_y = event.position.y - bounds.origin.y;
                    scrollbar.drag_state = Some(DragState {
                        thumb_offset_y: mouse_y - thumb_top,
                    });
                    cx.notify();
                });
            },
        )
}
