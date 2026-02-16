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
    div, img, prelude::*, pulsating_between, px, quadratic, rems, solid_background, Animation,
    AnimationExt, AppContext, AsyncApp, Entity, Fill, FontWeight, ImageSource, SharedString,
    StyleRefinement, Window,
};
use hacker_news_api::Item;
use log::error;
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
    pub fn new(
        app: &mut AsyncApp,
        content: Entity<ContentView>,
        item: Item,
        order_change: i64,
        rank: usize,
        comment_count_changed: i64,
    ) -> Entity<Self> {
        app.new(move |_cx| Self {
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
            comment_count_changed: comment_count_changed
                .gt(&0)
                .then(|| format!("+{comment_count_changed}").into()),
        })
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

        let ids = self.comment_ids.clone();
        let entity = cx.weak_entity();
        let content = self.content.clone();
        let content_close = self.content.clone();

        let article = cx.weak_entity().upgrade();
        let close_comment = cx.weak_entity();

        let comments_col = div()
            .w(rems(4.))
            .justify_end()
            .id("comments")
            .when_some(self.comment_count.as_ref(), |div, comments| {
                div.flex()
                    .cursor_pointer()
                    .rounded_md()
                    .on_click(move |_, _, app| {
                        let article = article.clone();
                        if let Err(err) = entity.update(app, |article: &mut ArticleView, _cx| {
                            article.loading_comments = true;
                        }) {
                            error!("Failed to set loading flag: {err}");
                        };

                        let entity = entity.clone();
                        let content = content.clone();
                        let ids = ids.clone();

                        app.spawn(async move |app: &mut AsyncApp| {
                            let client =
                                app.read_global(|client: &ApiClientState, _| client.0.clone());
                            let items = async_compat::Compat::new(
                                client.items(&ids).try_collect::<Vec<_>>(),
                            )
                            .await
                            .unwrap_or_default();

                            let comments = items
                                .into_iter()
                                .filter_map(|comment| {
                                    article.clone().map(|article| {
                                        CommentView::new(app, comment, article.clone())
                                    })
                                })
                                .collect();

                            if let Err(err) = entity.update(app, |this: &mut ArticleView, _cx| {
                                this.comments = comments;
                            }) {
                                error!("Failed to update comments: {err}");
                            };

                            if let Err(err) =
                                entity.update(app, |article: &mut ArticleView, _cx| {
                                    article.loading_comments = false;
                                })
                            {
                                error!("Failed to set loading comments: {err}");
                            };

                            content.update(app, |_content: &mut ContentView, cx| {
                                cx.emit(ContentEvent::ViewingComments(true));
                            });
                        })
                        .detach();
                    })
                    .hover(hover_element)
                    .flex()
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
            })
            .map(|div| {
                if let Some(new_comments_added) = self.comment_count_changed.clone() {
                    div.with_animation(
                        "comment_col",
                        Animation::new(Duration::from_secs(2)).with_easing(quadratic),
                        move |div, n| div.opacity(n),
                    )
                    .into_any()
                } else {
                    div.into_any()
                }
            });

        let url = self.url.clone();

        let weak_entity = cx.weak_entity();
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
                            } else if let Some(entity) = weak_entity.upgrade() {
                                let view = entity.read(app);
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
                                    if let Some(this) = close_comment.upgrade() {
                                        this.update(app, |article, _cx| {
                                            article.comments.clear();
                                        });
                                    }

                                    content_close.update(app, |_content: &mut ContentView, cx| {
                                        cx.emit(ContentEvent::ViewingComments(false));
                                    })
                                }),
                        )
                        .children(self.comments.clone()),
                )
            })
    }
}
