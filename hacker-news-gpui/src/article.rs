//! Article view.
use crate::{
    UrlHover,
    common::hover_element,
    common::{COMMENT_IMAGE, parse_date, url_punycode},
    content::{ContentEvent, ContentView},
    rich_text::{ViewStyledText, parse_layout},
    theme::Theme,
};
use gpui::{
    Animation, AnimationExt, AppContext, AsyncApp, Entity, Fill, ImageSource, SharedString,
    StyleRefinement, Window, div, img, prelude::*, pulsating_between, px, quadratic, rems, rgb,
};
use hacker_news_api::Item;
use std::{rc::Rc, sync::Arc, time::Duration};

// An article view is rendered for each article item.
pub struct ArticleView {
    /// The title of the article.
    title: SharedString,
    /// The author of the article, formatted as "by {author}".
    pub author: SharedString,
    /// The number of comments on the article, if available.
    pub comment_count: Option<SharedString>,
    /// The URL of the article, if available.
    url: Option<SharedString>,
    /// The label indicating the change in article order/rank.
    order_change_label: SharedString,
    /// The change in article order/rank.
    order_change: i64,
    /// The age of the article, formatted as a string.
    pub age: SharedString,
    /// The image source for the comment icon.
    comment_image: ImageSource,
    // The rank of the article, formatted as a string.
    // rank: SharedString,
    /// The IDs of the comments for this article.
    pub comment_ids: Arc<Vec<u64>>,
    /// The entity representing the content view associated with this article.
    pub content_entity: Entity<ContentView>,
    /// Whether the comments are currently loading.
    pub loading_comments: bool,
    /// The delta in comment count since the last update, if available.
    comment_count_changed: Option<SharedString>,
    /// Article id
    pub id: u64,
    /// Article body.
    pub article_text: Option<Rc<ViewStyledText>>,
}

impl ArticleView {
    /// Creates a new `ArticleView` entity for the given article item.
    ///
    /// # Arguments
    ///
    /// * `app` - The mutable reference to the async application context.
    /// * `content_entity` - The entity representing the content view.
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
        content_entity: Entity<ContentView>,
        item: Item,
        order_change: i64,
        // rank: usize,
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
                url: item.url.as_deref().map(url_punycode).map(Into::into),
                order_change_label: if order_change == 0 {
                    Default::default()
                } else {
                    format!("{order_change}").into()
                },
                order_change,
                age: parse_date(item.time).unwrap_or_default().into(),
                comment_image: ImageSource::Image(Arc::clone(&COMMENT_IMAGE)),
                // rank: format!("{rank}").into(),
                comment_ids: Arc::new(item.kids),
                content_entity,
                loading_comments: false,
                comment_count_changed: changed,
                id: item.id,
                article_text: item
                    .text
                    .as_deref()
                    .map(parse_layout)
                    .map(Into::into)
                    .map(Rc::new),
            }
        });

        // Set a timer to allow the comment count to return
        // after the the animation has completed.
        if comment_count_changed != 0 {
            let article_entity = article_entity.clone();
            app.spawn(async move |app: &mut AsyncApp| {
                app.background_executor()
                    .timer(Duration::from_secs(5))
                    .await;
                article_entity.update(app, |article: &mut ArticleView, cx| {
                    article.comment_count_changed = None;
                    cx.notify();
                });
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
    ///
    /// # Returns
    ///
    /// Returns a [`gpui::Stateful<gpui::Div>`] containing the rendered new comments cell.
    fn render_new_comments_cell(
        &self,
        div: gpui::Stateful<gpui::Div>,
        new_comments_added: &SharedString,
        article_entity: Entity<ArticleView>,
    ) -> gpui::Stateful<gpui::Div> {
        div.cursor_pointer()
            .on_click(self.fetch_comments_call_back(article_entity))
            .flex()
            .flex_row()
            .child(
                gpui::div()
                    .flex()
                    .flex_row()
                    .child(
                        gpui::div()
                            .bg(Fill::Color(rgb(0xFF69B4).into()))
                            .text_align(gpui::TextAlign::Center)
                            .rounded(rems(0.25))
                            .px(px(4.0))
                            .text_size(rems(0.75))
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
    /// # Returns
    ///
    /// Returns a [`gpui::Stateful<gpui::Div>`] containing the rendered comments cell.
    fn render_comments_cell(
        &self,
        hover_element: impl Fn(StyleRefinement) -> StyleRefinement,
        article_entity: Entity<ArticleView>,
        div: gpui::Stateful<gpui::Div>,
        comments: &SharedString,
    ) -> gpui::Stateful<gpui::Div> {
        div.flex()
            .cursor_pointer()
            .rounded_md()
            .on_click(self.fetch_comments_call_back(article_entity))
            .hover(hover_element)
            .flex_row()
            .child(gpui::div().text_size(rems(0.75)).child(comments.clone()))
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

    fn fetch_comments_call_back(
        &self,
        article_entity: Entity<ArticleView>,
    ) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static {
        let content_entity = self.content_entity.clone();

        move |_, _, app| {
            let article_entity = article_entity.clone();

            article_entity.update(app, |article_view: &mut ArticleView, _cx| {
                article_view.loading_comments = true;
            });

            content_entity.update(app, |_content_view: &mut ContentView, cx| {
                cx.emit(ContentEvent::OpenComments(article_entity))
            });
        }
    }
}

impl Render for ArticleView {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme: Theme = window.appearance().into();
        let viewing_article = self
            .content_entity
            .read(cx)
            .viewing_article_id
            .map(|id| id == self.id)
            .unwrap_or(false);

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

        let article_entity = cx.entity();

        let comments_col = div().id("comments").map(|div| {
            if let Some(new_comments_added) = self.comment_count_changed.as_ref() {
                self.render_new_comments_cell(div, new_comments_added, article_entity)
                    .into_any()
            } else if let Some(comments) = self.comment_count.as_ref() {
                self.render_comments_cell(hover_element(theme), article_entity, div, comments)
                    .into_any()
            } else if self.article_text.is_some() {
                // The article has a body but no comments. In place of the comment count
                // we'll add a character and click handler.
                div.id("text-open")
                    .cursor_pointer()
                    .hover(hover_element(theme))
                    .on_click(self.fetch_comments_call_back(article_entity))
                    .text_align(gpui::TextAlign::Center)
                    .rounded_md()
                    .child("*")
                    .into_any()
            } else {
                div.into_any()
            }
        });

        let url = self.url.clone();
        let article_entity = cx.entity();

        let load_comments_cb = (self.article_text.is_some() && url.is_none())
            .then(|| self.fetch_comments_call_back(article_entity.clone()));

        let title_col = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .child(
                div()
                    .id("title")
                    .p_1()
                    .w_full()
                    .child(self.title.clone())
                    .cursor_pointer()
                    .on_click(move |event, window, app| {
                        if let Some(url) = url.as_deref() {
                            app.open_url(url.as_ref());
                        } else if let Some(cb) = &load_comments_cb {
                            cb(event, window, app);
                        };
                    })
                    .on_hover(move |hover, _window, app| {
                        if !hover {
                            app.set_global::<UrlHover>(UrlHover(None));
                        } else {
                            let url = article_entity.read(app).url.clone();
                            app.set_global::<UrlHover>(UrlHover(url));
                        }
                    })
                    .hover(hover_element(theme)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .p_1()
                    .italic()
                    .text_size(rems(0.75))
                    .child(self.author.clone())
                    .child(self.age.clone())
                    .child(comments_col)
                    .gap_x(rems(0.1)),
            );

        div().w_full().child(
            div()
                .flex()
                .flex_row()
                .rounded_md()
                .shadow_md()
                .bg(theme.surface())
                .border_1()
                .border_color(theme.border())
                .when(self.order_change > 2, |div| {
                    div.text_color(theme.text_increasing())
                })
                .when(self.order_change < -2, |div| {
                    div.text_color(theme.text_decreasing())
                })
                .when(viewing_article, |div| div.opacity(0.75))
                .child(
                    div().overflow_hidden().child(
                        div()
                            .flex()
                            .flex_row()
                            .children([rank_change_col, title_col]),
                    ),
                ),
        )
    }
}
