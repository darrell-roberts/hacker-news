//! Render implementation for content view.
use super::ContentView;
use crate::theme::Theme;
use gpui::{
    App, AppContext, DefiniteLength, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Window, div, prelude::*, px, rems,
};

impl Render for ContentView {
    fn render(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme: Theme = window.appearance().into();

        let content_entity_1 = cx.entity();
        let content_entity_2 = cx.entity();

        let articles_scrollbar_dragging =
            cx.read_entity(&self.articles_scrollbar, |sb, _| sb.is_dragging());
        let comments_scrollbar_dragging =
            cx.read_entity(&self.comments_scrollbar, |sb, _| sb.is_dragging());
        let scrollbar_dragging = articles_scrollbar_dragging || comments_scrollbar_dragging;

        let articles_scrollbar_for_move = self.articles_scrollbar.clone();
        let comments_scrollbar_for_move = self.comments_scrollbar.clone();
        let articles_scrollbar_for_up = self.articles_scrollbar.clone();
        let comments_scrollbar_for_up = self.comments_scrollbar.clone();

        div()
            .id("content")
            .flex()
            .flex_row()
            .flex_shrink_0()
            .h_full()
            .min_h_0()
            // Only listen to move/up at the container level when actively dragging
            .when(self.is_dragging_divider, |div| {
                div.on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                    this.articles_width =
                        (event.position.x - this.divider_drag_offset).max(px(100.0));
                    cx.notify();
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _event, _window, cx| {
                        this.is_dragging_divider = false;
                        cx.notify();
                    }),
                )
                .cursor_col_resize()
            })
            // Handle scrollbar drags at the container level so they work
            // even when the mouse leaves the narrow scrollbar track.
            .when(scrollbar_dragging, |div| {
                div.on_mouse_move(
                    move |event: &MouseMoveEvent, _window: &mut Window, app: &mut App| {
                        if articles_scrollbar_dragging {
                            articles_scrollbar_for_move.update(app, |sb, cx| {
                                sb.handle_drag_move(event, cx);
                            });
                        }
                        if comments_scrollbar_dragging {
                            comments_scrollbar_for_move.update(app, |sb, cx| {
                                sb.handle_drag_move(event, cx);
                            });
                        }
                    },
                )
                .on_mouse_up(
                    MouseButton::Left,
                    move |_event: &MouseUpEvent, _window: &mut Window, app: &mut App| {
                        if articles_scrollbar_dragging {
                            articles_scrollbar_for_up.update(app, |sb, cx| {
                                sb.handle_drag_end(cx);
                            });
                        }
                        if comments_scrollbar_dragging {
                            comments_scrollbar_for_up.update(app, |sb, cx| {
                                sb.handle_drag_end(cx);
                            });
                        }
                    },
                )
            })
            .child(
                div()
                    .flex()
                    .flex_row()
                    .h_full()
                    .w(DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(
                        self.articles_width,
                    )))
                    .child(
                        div()
                            .id("articles")
                            .track_focus(&self.articles_focus_handle)
                            .track_scroll(&self.articles_scroll_handle)
                            .on_hover(move |hover, window, cx| {
                                let focus_handle = cx
                                    .read_entity(&content_entity_1, |content_view, _cx| {
                                        content_view.articles_focus_handle.clone()
                                    });
                                if *hover {
                                    window.focus(&focus_handle, cx);
                                } else {
                                    window.blur()
                                }
                            })
                            .h_full()
                            .overflow_y_scroll()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .pr_1()
                            .mr_1()
                            .pb_2()
                            .children(
                                self.articles
                                    .iter()
                                    .map(|article| div().w_full().m_1().child(article.clone())),
                            ),
                    )
                    .child(self.articles_scrollbar.clone()),
            )
            .child(
                div()
                    .id("divider")
                    .h_full()
                    .w(px(1.0))
                    .mt_4()
                    .flex_shrink_0()
                    .cursor_col_resize()
                    .bg(theme.border())
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                            this.divider_drag_offset = event.position.x - this.articles_width;
                            this.is_dragging_divider = true;
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                        if this.is_dragging_divider {
                            this.articles_width =
                                (event.position.x - this.divider_drag_offset).max(px(100.0));
                            cx.notify();
                        }
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, cx| {
                            this.is_dragging_divider = false;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .h_full()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .id("comments")
                            .track_focus(&self.comments_focus_handle)
                            .track_scroll(&self.comments_scroll_handle)
                            .on_hover(move |hover, window, cx| {
                                let focus_handle = cx
                                    .read_entity(&content_entity_2, |content_view, _cx| {
                                        content_view.comments_focus_handle.clone()
                                    });
                                if *hover {
                                    window.focus(&focus_handle, cx);
                                } else {
                                    window.blur();
                                }
                            })
                            .h_full()
                            .flex_col()
                            .min_w_0()
                            .overflow_y_scroll()
                            .flex_1()
                            .pb_2()
                            .ml_1()
                            .pr(px(8.0))
                            .when(self.fetching_comments, |div| {
                                div.flex()
                                    .justify_center()
                                    .items_center()
                                    .text_size(rems(1.5))
                                    .child("Fetching comments...")
                            })
                            .when(!self.fetching_comments, |div| {
                                self.render_comments(cx, theme, div)
                            }),
                    )
                    .child(self.comments_scrollbar.clone()),
            )
    }
}

impl ContentView {
    /// Renders opened comments.
    ///
    /// # Arguments
    ///
    /// * `cx` - Content view context.
    /// * `theme` - The current theme to use for styling.
    /// * `el` - The div element to render the comments into.
    ///
    /// # Returns
    ///
    /// Returns a [`gpui::Stateful<gpui::Div>`] containing the rendered comments section.
    fn render_comments(
        &self,
        cx: &mut gpui::Context<ContentView>,
        theme: Theme,
        el: gpui::Stateful<gpui::Div>,
    ) -> gpui::Stateful<gpui::Div> {
        let comment_entities = self.comment_entities.clone();
        let content_entity = cx.entity();

        // If we don't have either an article body or any comments to show then we have nothing
        // to render.
        if self.article_body_view.is_none() && self.comment_entities.is_empty() {
            return el;
        }

        el.child(
            div()
                .bg(theme.bg())
                .rounded_tl_md()
                .pb_2()
                .child(
                    div()
                        .flex()
                        .flex_grow()
                        .flex_row()
                        .text_size(rems(0.75))
                        .bg(theme.comment_border())
                        .rounded_tl_md()
                        .child(div().pl_1().child("[X]"))
                        .cursor_pointer()
                        .id("close-comments")
                        .on_click(move |_event, _window, app| {
                            // Clear any open comments for another article
                            content_entity.update(app, |content_view, _cx| {
                                content_view.comment_entities.clear();
                                content_view.viewing_article_id = None;
                            });
                        }),
                )
                .when_some(self.article_body_view.as_ref(), |div, view_styled_text| {
                    div.child(view_styled_text.clone())
                })
                .children(comment_entities.clone()),
        )
    }
}
