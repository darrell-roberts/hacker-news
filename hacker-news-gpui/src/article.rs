//! Article view.
use crate::{
    comment::CommentView,
    common::{parse_date, COMMENT_IMAGE},
    content::{ContentEvent, ContentView},
    theme::Theme,
    ApiClientState, UrlHover,
};
use futures::TryStreamExt;
use gpui::{
    div, img, prelude::*, pulsating_between, px, quadratic, rems, rgb, solid_background, Animation,
    AnimationExt, AppContext, AsyncApp, Entity, Fill, FontWeight, ImageSource, SharedString,
    StyleRefinement, Window,
};
use hacker_news_api::Item;
use std::{sync::Arc, time::Duration};

// An article view is rendered for each article item.
pub struct ArticleView {
    title: SharedString,
    author: SharedString,
    comment_count: Option<SharedString>,
    url: Option<SharedString>,
    order_change_label: SharedString,
    order_change: i64,
    age: SharedString,
    comment_image: ImageSource,
    rank: SharedString,
    comments: Vec<Entity<CommentView>>,
    comment_ids: Arc<Vec<u64>>,
    content: Entity<ContentView>,
    loading_comments: bool,
    comment_count_changed: Option<SharedString>,
}

impl ArticleView {
    /// Creates a new `ArticleView` entity for the given article item.
    ///
    /// # Arguments
    ///
    /// * `app` - The mutable reference to the async application context.
    /// * `content` - The entity representing the content view.
    /// * `item` - The Hacker News article item to render.
    /// * `order_change` - The change in article order/rank.
    /// * `rank` - The current rank of the article.
    /// * `comment_count_changed` - The delta in comment count since last update.
    ///
    /// # Returns
    ///
    /// An entity representing the newly created `ArticleView`.
    pub fn new(
        app: &mut AsyncApp,
        content: Entity<ContentView>,
        item: Item,
        order_change: i64,
        rank: usize,
        comment_count_changed: i64,
    ) -> Entity<Self> {
        let article_entity = app.new(|_cx| {
            let changed = if comment_count_changed.is_negative() {
                Some(format!("{comment_count_changed}"))
            } else if comment_count_changed > 0 {
                Some(format!("+{comment_count_changed}"))
            } else {
                None
            }
            .map(Into::into);

            Self {
                title: item.title.unwrap_or_default().into(),
                author: format!("by {}", item.by.clone()).into(),
                comment_count: item
                    .descendants
                    .filter(|&n| n > 0)
                    .map(|n| format!("{n}"))
                    .map(Into::into),
                url: item.url.map(Into::into),
                order_change_label: if order_change == 0 {
                    Default::default()
                } else {
                    format!("{order_change}").into()
                },
                order_change,
                age: parse_date(item.time).unwrap_or_default().into(),
                comment_image: ImageSource::Image(Arc::clone(&COMMENT_IMAGE)),
                rank: format!("{rank}").into(),
                comments: Vec::new(),
                comment_ids: Arc::new(item.kids),
                content,
                loading_comments: false,
                comment_count_changed: changed,
            }
        });

        let entity = article_entity.downgrade();
        if comment_count_changed > 0 {
            app.spawn(async move |app: &mut AsyncApp| {
                app.background_executor()
                    .timer(Duration::from_secs(5))
                    .await;
                if let Some(entity) = entity.upgrade() {
                    entity.update(app, |article: &mut ArticleView, _| {
                        article.comment_count_changed = None;
                    });
                }
            })
            .detach();
        }

        article_entity
    }

    /// Render the comments cell when we have a new total comments delta.
    ///
    /// # Arguments
    ///
    /// * `div` - The div element to render the new comments cell into.
    /// * `new_comments_added` - The shared string representing the number of new comments added.
    fn render_new_comments_cell(
        &self,
        div: gpui::Stateful<gpui::Div>,
        new_comments_added: &SharedString,
    ) -> gpui::Stateful<gpui::Div> {
        div.flex().flex_row().child(
            gpui::div()
                .flex()
                .flex_row()
                .child(
                    gpui::div()
                        .bg(Fill::Color(rgb(0xFF69B4).into()))
                        .text_align(gpui::TextAlign::Center)
                        .rounded(rems(0.25))
                        .child(new_comments_added.clone())
                        .with_animation(
                            "comment-count-changed-fade",
                            Animation::new(Duration::from_secs(5)).with_easing(quadratic),
                            |el, n| el.opacity(1.0 - n),
                        ),
                )
                .child(gpui::div().child(img(self.comment_image.clone()))),
        )
    }

    /// Render the comments cell. This shows the total number of comments next
    /// to an actionable comment icon.
    ///
    /// # Arguments
    ///
    /// * `hover_element` - A function that takes a `StyleRefinement` and returns a refined style, used for hover effects.
    /// * `article_entity` - The entity representing the article view.
    /// * `div` - The div element to render the comments cell into.
    /// * `comments` - The shared string representing the number of comments.
    ///
    /// This shows the total number of comments next to an actionable comment icon.
    fn render_comments_cell(
        &self,
        hover_element: impl Fn(StyleRefinement) -> StyleRefinement,
        article_entity: Entity<ArticleView>,
        div: gpui::Stateful<gpui::Div>,
        comments: &SharedString,
    ) -> gpui::Stateful<gpui::Div> {
        let ids = self.comment_ids.clone();
        let content = self.content.clone();

        div.flex()
            .cursor_pointer()
            .rounded_md()
            .on_click(move |_, _, app| {
                article_entity.update(app, |article: &mut ArticleView, _cx| {
                    article.loading_comments = true;
                });

                let article_entity = article_entity.clone();
                let content = content.clone();
                let ids = ids.clone();

                app.spawn(async move |app: &mut AsyncApp| {
                    let client = app.read_global(|client: &ApiClientState, _| client.0.clone());
                    let items =
                        async_compat::Compat::new(client.items(&ids).try_collect::<Vec<_>>())
                            .await
                            .unwrap_or_default();

                    let comments = items
                        .into_iter()
                        .map(|comment| CommentView::new(app, comment, article_entity.clone()))
                        .collect();

                    article_entity.update(app, |this: &mut ArticleView, _cx| {
                        this.comments = comments;
                        this.loading_comments = false;
                    });

                    content.update(app, |_content: &mut ContentView, cx| {
                        cx.emit(ContentEvent::ViewingComments(true));
                    });
                })
                .detach();
            })
            .hover(hover_element)
            .flex_row()
            .child(comments.clone())
            .child(gpui::div().child(img(self.comment_image.clone())).when(
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
            ))
    }

    /// Renders opened comments.
    ///
    /// Renders opened comments.
    ///
    /// # Arguments
    ///
    /// * `theme` - The current theme to use for styling.
    /// * `article_entity` - The entity representing the article view.
    /// * `content_close` - The entity representing the content view to close comments.
    /// * `el` - The div element to render the comments into.
    fn render_comments(
        &mut self,
        theme: Theme,
        article_entity: Entity<ArticleView>,
        content_close: Entity<ContentView>,
        el: gpui::Div,
    ) -> gpui::Div {
        el.child(
            div()
                .bg(theme.comment_border())
                .mt_1()
                .ml_1()
                .pl_1()
                .rounded_tl_md()
                .child(
                    div()
                        .flex()
                        .flex_grow()
                        .flex_row()
                        .text_size(rems(0.75))
                        .child("[X]")
                        .cursor_pointer()
                        .id("close-comments")
                        .on_click(move |_event, _window, app| {
                            article_entity.update(app, |article, _cx| {
                                article.comments.clear();
                            });

                            content_close.update(app, |_content: &mut ContentView, cx| {
                                cx.emit(ContentEvent::ViewingComments(false));
                            })
                        }),
                )
                .children(self.comments.clone()),
        )
    }
}

impl Render for ArticleView {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme: Theme = window.appearance().into();

        let rank_change_col = div()
            .flex()
            .italic()
            .text_size(rems(0.75))
            .w(rems(0.75))
            .when(
                self.order_change.is_positive() && self.order_change > 0,
                |div| div.text_color(theme.text_increasing()),
            )
            .when(self.order_change.is_negative(), |div| {
                div.text_color(theme.text_decreasing())
            })
            .justify_end()
            .items_center()
            .child(self.order_change_label.clone());

        let hover_element = |style: StyleRefinement| {
            style
                .font_weight(FontWeight::BOLD)
                .bg(Fill::Color(solid_background(theme.hover())))
        };

        let article_entity = cx.entity();

        let comments_col = div().w(rems(4.)).justify_end().id("comments").map(|div| {
            if let Some(new_comments_added) = self.comment_count_changed.as_ref() {
                self.render_new_comments_cell(div, new_comments_added)
            } else if let Some(comments) = self.comment_count.as_ref() {
                self.render_comments_cell(hover_element, article_entity, div, comments)
            } else {
                div
            }
        });

        let url = self.url.clone();
        let article_entity = cx.entity();

        let title_col = div()
            .flex()
            .flex_row()
            .flex_grow()
            .child(
                div().rounded_md().child(
                    div()
                        .id("title")
                        .child(self.title.clone())
                        .cursor_pointer()
                        .on_click(move |_, _, app| {
                            if let Some(url) = url.as_deref() {
                                app.open_url(url.as_ref());
                            }
                        })
                        .on_hover(move |hover, _window, app| {
                            if !hover {
                                app.set_global::<UrlHover>(UrlHover(None));
                            } else {
                                let view = article_entity.read(app);
                                app.set_global::<UrlHover>(UrlHover(view.url.clone()));
                            }
                        })
                        .hover(hover_element),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .italic()
                    .items_center()
                    .text_size(rems(0.75))
                    .child(self.author.clone())
                    .child(self.age.clone())
                    .gap_x(px(5.0)),
            )
            .gap_x(px(5.0));

        let article_entity = cx.entity();
        let content_close = self.content.clone();

        div()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .rounded_md()
                    .shadow_sm()
                    .bg(theme.surface())
                    .border_1()
                    .border_color(theme.border())
                    .when(self.order_change > 2, |div| {
                        div.text_color(theme.text_increasing())
                    })
                    .when(self.order_change < -2, |div| {
                        div.text_color(theme.text_decreasing())
                    })
                    .child(
                        div().m_1().child(
                            div()
                                .flex()
                                .flex_row()
                                .children([
                                    rank_change_col,
                                    div()
                                        .w(rems(2.))
                                        .text_align(gpui::TextAlign::Right)
                                        .child(self.rank.clone()),
                                    div().child(comments_col),
                                    title_col,
                                ])
                                .gap_1(),
                        ),
                    ),
            )
            .when(!self.comments.is_empty(), |el| {
                self.render_comments(theme, article_entity, content_close, el)
            })
    }
}
