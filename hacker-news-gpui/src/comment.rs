//! Render comment
use crate::{
    article::ArticleView,
    common::{COMMENT_IMAGE, comment_entities, hover_element, parse_date},
    rich_text::{ParsedStyledText, TextLayout, parse_layout, rich_text_runs, url_ranges},
    theme::Theme,
};
use gpui::{
    Animation, AnimationExt as _, AppContext as _, AsyncApp, Entity, ImageSource,
    InteractiveElement, InteractiveText, ParentElement, Render, SharedString,
    StatefulInteractiveElement, StyleRefinement, Styled, StyledText, Window, div, img,
    prelude::FluentBuilder as _, pulsating_between, rems,
};
use hacker_news_api::Item;
use std::{sync::Arc, time::Duration};

/// Comment view with state.
pub struct CommentView {
    /// The comment text.
    text: SharedString,
    /// The author of the article, formatted as "by {author}".
    author: SharedString,
    /// The entities representing the child comments for this comment.
    children: Vec<Entity<CommentView>>,
    /// The ids of child comments.
    comment_child_ids: Arc<Vec<u64>>,
    /// The image source for the comment icon.
    comment_image: ImageSource,
    /// The number of comments on the comment, if available.
    comment_count: SharedString,
    /// Whether the comments are currently loading.
    loading_comments: bool,
    /// The top level article entity this comment is a descendant of.
    article_entity: Entity<ArticleView>,
    /// Text layout structure of the comment body for rendering as [`StyledText`].
    text_layout: Vec<TextLayout>,
    /// The age of the comment, formatted as a string.
    age: SharedString,
    /// Any urls that are in the comment body.
    urls: Vec<String>,
    /// Comment id.
    id: u64,
}

impl CommentView {
    /// Create a new comment view
    ///
    /// # Arguments
    ///
    /// * `cx` - The async application context used to create new entities.
    /// * `item` - The Hacker News API item representing the comment.
    /// * `article_entity` - The entity representing the parent article view.
    pub fn new(cx: &mut AsyncApp, item: Item, article_entity: Entity<ArticleView>) -> Entity<Self> {
        let ParsedStyledText { text, layout, urls } =
            item.text.as_deref().map(parse_layout).unwrap_or_default();

        cx.new(|_cx| Self {
            text: text.into(),
            author: format!("by: {} ({})", item.by, item.id).into(),
            children: Vec::new(),
            comment_count: format!("{}", item.kids.len()).into(),
            comment_child_ids: Arc::new(item.kids),
            comment_image: ImageSource::Image(Arc::clone(&COMMENT_IMAGE)),
            loading_comments: false,
            article_entity,
            text_layout: layout,
            urls,
            age: parse_date(item.time).unwrap_or_default().into(),
            id: item.id,
        })
    }

    /// Renders the comment text.
    ///
    /// # Arguments
    ///
    /// * `theme` - The current theme used for styling.
    /// * `comment_entity` - The entity representing this comment view.
    ///
    /// # Returns
    ///
    /// Returns a `gpui::Div` element containing the rendered comment text area UI.
    fn render_text_area(&self, theme: Theme, comment_entity: Entity<CommentView>) -> gpui::Div {
        div().p_1().child(
            InteractiveText::new(
                "comment_text",
                StyledText::new(self.text.clone())
                    .with_runs(rich_text_runs(theme, &self.text_layout).collect()),
            )
            .on_click(url_ranges(&self.text_layout), move |index, _window, app| {
                comment_entity.read_with(app, |this: &CommentView, app| {
                    if let Some(url) = this.urls.get(index) {
                        app.open_url(url);
                    }
                })
            }),
        )
    }

    /// Renders the comment footer with child comment count, author and date/time.
    ///
    /// # Arguments
    ///
    /// * `theme` - The current theme used for styling.
    /// * `comment_ids` - The list of child comment IDs.
    /// * `comment_entity` - The entity representing this comment view.
    ///
    /// # Returns
    ///
    /// Returns a `gpui::Div` element containing the comment footer UI.
    fn render_comment_footer(
        &self,
        theme: Theme,
        comment_ids: Arc<Vec<u64>>,
        comment_entity: Entity<CommentView>,
    ) -> gpui::Div {
        let id = self.id;

        gpui::div()
            .flex()
            .flex_row()
            .italic()
            .gap_1()
            .border_t_1()
            .p_1()
            .border_color(theme.border())
            .text_size(rems(0.75))
            .child(
                gpui::div()
                    .id("comment_id")
                    .child(self.author.clone())
                    .cursor_pointer()
                    .rounded_md()
                    .on_click(move |_event, _window, cx| {
                        cx.open_url(&format!("https://news.ycombinator.com/item?id={id}"));
                    })
                    .hover(hover_element(theme)),
            )
            .child(self.age.clone())
            .when(!self.comment_child_ids.is_empty(), |div| {
                self.render_child_comments(comment_ids, comment_entity, div, hover_element(theme))
            })
    }

    /// Render child comments that have opened.
    ///
    /// # Arguments
    ///
    /// * `comment_ids` - The list of child comment IDs.
    /// * `comment_entity` - The entity representing this comment view.
    /// * `el` - The parent Div element to which child comments UI will be attached.
    ///
    /// # Returns
    ///
    /// Returns a `gpui::Div` element containing the child comments UI.
    fn render_child_comments(
        &self,
        comment_ids: Arc<Vec<u64>>,
        comment_entity: Entity<CommentView>,
        el: gpui::Div,
        hover: impl Fn(StyleRefinement) -> StyleRefinement,
    ) -> gpui::Div {
        let article_entity = self.article_entity.clone();

        el.child(
            div()
                .id("child-comments")
                .cursor_pointer()
                .rounded_md()
                .hover(hover)
                .on_click(move |_event, _window, app| {
                    comment_entity.update(app, |this, _cx| {
                        this.loading_comments = true;
                    });

                    let comment_ids = comment_ids.clone();
                    let comment_entity = comment_entity.clone();
                    let article_entity = article_entity.clone();

                    app.spawn(async move |async_app| {
                        let comment_entities =
                            comment_entities(async_app, article_entity, &comment_ids).await;
                        comment_entity.update(async_app, |this, _cx| {
                            this.children = comment_entities;
                            this.loading_comments = false;
                        });
                    })
                    .detach();
                })
                .flex()
                .flex_row()
                .child(self.comment_count.clone())
                .child(div().child(img(self.comment_image.clone())).when(
                    self.loading_comments,
                    |el| {
                        gpui::div().child(
                            el.with_animation(
                                "comment-loading",
                                Animation::new(Duration::from_secs(1))
                                    .repeat()
                                    .with_easing(pulsating_between(0.1, 0.8)),
                                |label, delta| label.opacity(delta),
                            ),
                        )
                    },
                )),
        )
    }
}

impl Render for CommentView {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme: Theme = window.appearance().into();

        let comment_ids = self.comment_child_ids.clone();
        let comment_entity = cx.entity();

        div()
            .bg(theme.surface())
            .rounded_tl_md()
            .flex_col()
            .w_full()
            .mb_2()
            .shadow_md()
            .child(self.render_text_area(theme, comment_entity.clone()))
            .child(self.render_comment_footer(theme, comment_ids, comment_entity.clone()))
            .when(!self.children.is_empty(), |el| {
                el.child(
                    div()
                        .bg(theme.bg())
                        .ml_1()
                        .w_full()
                        .flex_col()
                        .rounded_tl_md()
                        .border_x_1()
                        .border_color(theme.comment_border())
                        .child(
                            div()
                                .bg(theme.comment_border())
                                .flex()
                                .flex_grow()
                                .flex_row()
                                .rounded_tl_md()
                                .text_size(rems(0.75))
                                .child(div().pl_1().child("[X]"))
                                .cursor_pointer()
                                .id("close-comments")
                                .on_click(move |_event, _window, app| {
                                    comment_entity.update(app, |comment_view, _cx| {
                                        comment_view.children.clear();
                                    });
                                }),
                        )
                        .children(self.children.clone()),
                )
            })
    }
}
