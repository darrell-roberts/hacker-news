//! Main content view
use crate::{
    ArticleSelection, article::ArticleView, article_body::ArticleBodyView, comment::CommentView,
    common::comment_entities, scrollbar::Scrollbar,
};
use background::{
    BackGroundError, restart_background_task, start_background_article_list_subscription,
    start_background_subscriptions,
};
use futures::channel;
use gpui::{
    App, AppContext, Entity, EventEmitter, FocusHandle, ListState, Pixels, ScrollHandle, Window,
    prelude::*, px,
};
use hacker_news_api::{ArticleType, Item};
use log::{error, info};
use std::{collections::HashMap, f32};

mod background;
mod render;

// Main content view.
pub struct ContentView {
    /// List of article view entities currently displayed.
    articles: Vec<Entity<ArticleView>>,
    /// State for scrolling and alignment of the article list.
    list_state: ListState,
    /// Tracks the ranking of each article so that if it moves up
    /// or down we can show by how much.
    article_ranks: HashMap<u64, usize>,
    /// Tracks the number of comments for an article so that when it
    /// changes we can show a visual indicator.
    article_comment_counts: HashMap<u64, u64>,
    /// Subscription to server side events is online.
    pub online: bool,
    /// Handle to the background task that updates articles.
    background_task: Option<gpui::Task<()>>,
    /// Sender channel for pushing article updates from background to foreground.
    article_sender: Option<channel::mpsc::Sender<Result<Vec<Item>, BackGroundError>>>,
    /// The number of times we have refresh due to an http server side event.
    background_refresh_count: usize,
    /// The entities representing the comments for this article.
    comment_entities: Vec<Entity<CommentView>>,
    /// The width of the article column, user adjustable.
    articles_width: Pixels,
    /// True when the user is adjusting the article column width using the divider.
    is_dragging_divider: bool,
    /// The offset between the mouse x position and the articles_width when
    /// the divider drag started. Applied during the drag so the divider
    /// stays exactly under the cursor.
    divider_drag_offset: Pixels,
    /// Fetching comments.
    fetching_comments: bool,
    /// Scroll handle for articles column.
    articles_scroll_handle: ScrollHandle,
    /// Scroll handle for comments column.
    comments_scroll_handle: ScrollHandle,
    /// Focus handle for articles
    articles_focus_handle: FocusHandle,
    /// Focus handle for comments
    comments_focus_handle: FocusHandle,
    /// Scrollbar entity for articles column.
    articles_scrollbar: Entity<Scrollbar>,
    /// Scrollbar entity for comments column.
    comments_scrollbar: Entity<Scrollbar>,
    /// Viewing Article text
    article_body_view: Option<Entity<ArticleBodyView>>,
    /// Viewing article id.
    pub viewing_article_id: Option<u64>,
}

/// Events emitted by the ContentView to signal UI updates or errors.
pub enum ContentEvent {
    /// Indicates the total number of articles currently displayed.
    TotalArticles(usize),
    /// Indicates the total number of refreshes due to background updates.
    TotalRefreshes(usize),
    /// Indicates an error, optionally containing an error message.
    Error(Option<String>),
    /// Check if we need to restart background.
    Terminated(ArticleType),
    /// Toggle online status
    OnlineToggle(bool),
    /// Open Comments
    OpenComments(Entity<ArticleView>),
}

impl EventEmitter<ContentEvent> for ContentView {}

impl ContentView {
    /// Create a new content view.
    ///
    /// # Arguments
    ///
    /// * `_window` - A mutable reference to the window instance.
    /// * `app` - A mutable reference to the application instance.
    ///
    /// # Returns
    ///
    /// Returns an [`Entity<ContentView>`] representing the newly created content view.
    pub fn new(_window: &mut Window, app: &mut App) -> Entity<Self> {
        let articles_focus_handle = app.focus_handle();
        let comments_focus_handle = app.focus_handle();

        let articles_scroll_handle = ScrollHandle::new();
        let comments_scroll_handle = ScrollHandle::new();

        let articles_scrollbar =
            Scrollbar::new(app, "articles_scrollbar", articles_scroll_handle.clone());
        let comments_scrollbar =
            Scrollbar::new(app, "comments_scrollbar", comments_scroll_handle.clone());

        let entity_content = app.new(|cx: &mut Context<Self>| {
            cx.subscribe_self(|content_view, event, cx| match event {
                ContentEvent::OnlineToggle(enable) => {
                    // There is no change here.
                    if content_view.online == *enable {
                        return;
                    }

                    if *enable {
                        restart_background_task(content_view, cx);
                    } else if let Some(task) = content_view.background_task.take() {
                        drop(task);
                        content_view.online = false;
                    }
                }
                ContentEvent::Terminated(terminated_category) => {
                    let current_category = cx.global::<ArticleSelection>().viewing_article_type;
                    info!("Terminated: {terminated_category} current: {current_category}");
                    if terminated_category == &current_category {
                        // We need to restart.
                        info!("Restarting background subscription for {terminated_category}");
                        restart_background_task(content_view, cx);
                    }
                }
                ContentEvent::OpenComments(article_entity) => {
                    content_view.fetching_comments = true;
                    let id = article_entity.read(cx).id;
                    let article_entity = article_entity.clone();
                    let comment_ids = article_entity.read(cx).comment_ids.clone();

                    // Create the article body view.
                    content_view.article_body_view =
                        article_entity.update(cx, |article_view, cx| {
                            article_view.article_text.as_ref().map(|styled_text| {
                                ArticleBodyView::new(
                                    cx,
                                    styled_text.clone(),
                                    article_view.author.clone(),
                                    article_view.age.clone(),
                                )
                            })
                        });

                    content_view.viewing_article_id = Some(id);
                    cx.notify();

                    cx.spawn(async move |content_entity, async_app| {
                        let comment_entities =
                            comment_entities(async_app, article_entity.clone(), &comment_ids).await;

                        async_app.update(|app| {
                            article_entity.update(app, |article_view, cx| {
                                article_view.loading_comments = false;
                                cx.notify();
                            });

                            if let Err(err) = content_entity.update(app, |content_view, cx| {
                                content_view.comment_entities = comment_entities;
                                content_view.fetching_comments = false;
                                cx.notify();
                            }) {
                                error!("Content view is gone: {err}");
                            }
                        });
                    })
                    .detach();
                }
                ContentEvent::Error(_)
                | ContentEvent::TotalArticles(_)
                | ContentEvent::TotalRefreshes(_) => (),
            })
            .detach();

            let list_state = ListState::new(0, gpui::ListAlignment::Top, px(5.0));

            cx.observe_keystrokes(|content_view, event, window, cx| {
                let articles_active = content_view.articles_focus_handle.is_focused(window);
                let comments_active = content_view.comments_focus_handle.is_focused(window);

                if articles_active || comments_active {
                    let handle = if articles_active {
                        &mut content_view.articles_scroll_handle
                    } else {
                        &mut content_view.comments_scroll_handle
                    };

                    match event.keystroke.key.as_str() {
                        "home" => {
                            scroll_handle(handle, cx, Direction::Up, px(f32::MAX));
                            cx.notify();
                        }
                        "end" => {
                            scroll_handle(handle, cx, Direction::Down, px(f32::MAX));
                            cx.notify();
                        }
                        "pageup" => {
                            scroll_handle(handle, cx, Direction::Up, handle.bounds().size.height);
                        }
                        "pagedown" => {
                            scroll_handle(handle, cx, Direction::Down, handle.bounds().size.height);
                        }
                        "up" => {
                            scroll_handle(handle, cx, Direction::Up, px(10.0));
                        }
                        "down" => {
                            scroll_handle(handle, cx, Direction::Down, px(10.0));
                        }
                        _ => {}
                    }
                }
            })
            .detach();

            cx.observe_global::<ArticleSelection>(move |content_view, cx| {
                let selection = *cx.global::<ArticleSelection>();
                // Reset ranks when we change selection.
                content_view.article_ranks.clear();
                // Remove viewing article body.
                content_view.article_body_view = None;
                match content_view.article_sender.as_ref() {
                    Some(tx) => {
                        info!("Opening stream for {selection:?}");
                        let old_task = content_view
                            .background_task
                            .replace(start_background_article_list_subscription(cx, tx.clone()));
                        if let Some(old_task) = old_task {
                            info!("dropping old task");
                            drop(old_task);
                        }
                    }
                    None => {
                        error!("No article sender on content view");
                    }
                }
                cx.notify();
            })
            .detach();

            Self {
                list_state,
                articles: Default::default(),
                article_ranks: Default::default(),
                online: false,
                background_task: None,
                article_sender: None,
                article_comment_counts: Default::default(),
                background_refresh_count: 0,
                comment_entities: Vec::new(),
                articles_width: px(800.0),
                is_dragging_divider: false,
                divider_drag_offset: px(0.0),
                fetching_comments: false,
                articles_scroll_handle,
                comments_scroll_handle,
                articles_focus_handle,
                comments_focus_handle,
                articles_scrollbar,
                comments_scrollbar,
                article_body_view: None,
                viewing_article_id: None,
            }
        });

        let background_task = start_background_subscriptions(app, &entity_content);
        entity_content.update(app, |content_view, ctx| {
            content_view.background_task = Some(background_task);
            content_view.online = true;
            ctx.emit(ContentEvent::OnlineToggle(true));
        });
        entity_content
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Down,
}

/// Apply scrolling to a scroll handle.
fn scroll_handle(
    handle: &mut ScrollHandle,
    cx: &mut Context<'_, ContentView>,
    direction: Direction,
    distance: Pixels,
) {
    let mut offset = handle.offset();
    offset.y = match direction {
        Direction::Up => offset.y + distance,
        Direction::Down => offset.y - distance,
    };
    handle.set_offset(offset);
    cx.notify();
}
