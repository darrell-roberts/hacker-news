//! Main content view
use crate::{
    ApiClientState, ArticleSelection, article::ArticleView, comment::CommentView,
    common::comment_entities, scrollbar::Scrollbar, theme::Theme,
};
use async_compat::Compat;
use futures::{SinkExt, StreamExt, TryStreamExt as _, channel};
use gpui::{
    App, AppContext, DefiniteLength, Entity, EventEmitter, FocusHandle, ListState, MouseButton,
    MouseMoveEvent, MouseUpEvent, Pixels, ScrollHandle, Window, div, prelude::*, px, rems,
};
use hacker_news_api::{ArticleType, Item, subscribe_to_article_list};
use log::{error, info};
use std::{collections::HashMap, f32};

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
    pub background_task: Option<gpui::Task<()>>,
    /// Sender channel for pushing article updates from background to foreground.
    pub article_sender: Option<channel::mpsc::Sender<Result<Vec<Item>, BackGroundError>>>,
    /// The number of times we have refresh due to an http server side event.
    pub background_refresh_count: usize,
    /// The entities representing the comments for this article.
    pub comment_entities: Vec<Entity<CommentView>>,
    /// The width of the article column, user adjustable.
    articles_width: Pixels,
    /// True when the user is adjusting the article column width using the divider.
    is_dragging_divider: bool,
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

                    // Toggle viewing for articles that are no longer viewing comments.
                    for article in &content_view.articles {
                        let viewing = article.read(cx).viewing_comments;
                        let article_id = article.read(cx).id;

                        if viewing && (article_id != id) {
                            article.update(cx, |article_view, _cx| {
                                article_view.viewing_comments = false;
                            });
                        }
                    }

                    cx.spawn(async move |weak_content_view_entity, async_app| {
                        let comment_entities =
                            comment_entities(async_app, article_entity, &comment_ids).await;
                        async_app.update(|app| {
                            if let Err(err) =
                                weak_content_view_entity.update(app, |content_view, _cx| {
                                    content_view.comment_entities = comment_entities;
                                    content_view.fetching_comments = false;
                                })
                            {
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
                fetching_comments: false,
                articles_scroll_handle,
                comments_scroll_handle,
                articles_focus_handle,
                comments_focus_handle,
                articles_scrollbar,
                comments_scrollbar,
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

/// Restarts the background task by dropping the current task and replacing it with a new one.
fn restart_background_task(content_view: &mut ContentView, cx: &mut Context<'_, ContentView>) {
    if let Some(tx) = content_view.article_sender.as_ref()
        && !tx.is_closed()
    {
        let task = start_background_article_list_subscription(cx, tx.clone());
        content_view.background_task.replace(task);
        content_view.online = true;
        cx.emit(ContentEvent::OnlineToggle(true));
    }
}

#[derive(Debug, Clone)]
/// Background event subscription errors.
pub enum BackGroundError {
    /// The background task for a given category has been terminated.
    Terminated(ArticleType),
    /// A Running event failed to update existing items with provided error message.
    EventFailed(String),
}

/// Starts a background task that subscribes to the top stories stream,
/// fetches article data, and updates the Content entity accordingly.
///
/// This function is intended to be called once when initializing the Content view.
///
/// This function sets up two asynchronous tasks:
/// 1. One task listens for new batches of articles from a channel and updates
///    the Content entity's articles, list state, and ranking map, unless the
///    user is currently viewing comments.
/// 2. Another task subscribes to the top stories stream, fetches the latest
///    articles using the API client, and sends them through the channel to be
///    processed by the first task.
///
/// # Arguments
///
/// * `app` - A mutable reference to the application instance.
/// * `entity_content` - The entity representing the Content view to be updated.
///
/// # Returns
///
/// Returns a [`gpui::Task<()>`] representing the spawned background task.
fn start_background_subscriptions(
    app: &mut App,
    entity_content: &Entity<ContentView>,
) -> gpui::Task<()> {
    let entity_content = entity_content.clone();
    let (tx, mut rx) = channel::mpsc::channel::<Result<Vec<Item>, BackGroundError>>(10);

    // Keep a reference to the send channel so we can restart the background
    // if we lose connection.
    entity_content.update(app, |entity_view, _cx| {
        entity_view.article_sender.replace(tx.clone());
    });

    app.spawn(async move |app| {
        while let Some(items) = rx.next().await {
            match items {
                Ok(items) => {
                    let viewing_id = app.read_entity(&entity_content, |content_view, app| {
                        content_view.articles.iter().find_map(|article_entity| {
                            let article_view = article_entity.read(app);
                            article_view.viewing_comments.then_some(article_view.id)
                        })
                    });

                    let current_ranking_map = items
                        .iter()
                        .enumerate()
                        .map(|(index, item)| (item.id, index))
                        .collect::<HashMap<_, _>>();

                    let current_comment_counts = items
                        .iter()
                        .map(|item| (item.id, item.descendants.unwrap_or(0)))
                        .collect::<HashMap<_, _>>();

                    // Create an ArticleView for each item.
                    let views = items
                        .into_iter()
                        .enumerate()
                        .map(|(index, article)| {
                            let order_change = app.read_entity(&entity_content, |content, _app| {
                                match content.article_ranks.get(&article.id) {
                                    Some(rank) => (*rank as i64) - (index as i64),
                                    None => 0,
                                }
                            });

                            let last_comment_count =
                                app.read_entity(&entity_content, |content, _app| {
                                    content.article_comment_counts.get(&article.id).cloned()
                                });

                            let background_refresh_count = app
                                .read_entity(&entity_content, |content, _app| {
                                    content.background_refresh_count
                                });

                            let comment_count_changed: i64 = if background_refresh_count > 0 {
                                let last_comment_count = last_comment_count.unwrap_or(0) as i64;
                                let current_comment_count = article.descendants.unwrap_or(0) as i64;

                                if last_comment_count > 0 && current_comment_count > 0 {
                                    current_comment_count - last_comment_count
                                } else {
                                    0
                                }
                            } else {
                                0
                            };

                            let viewing_comments =
                                viewing_id.map(|id| id == article.id).unwrap_or(false);

                            ArticleView::new(
                                app,
                                entity_content.clone(),
                                article,
                                order_change,
                                index + 1,
                                comment_count_changed,
                                viewing_comments,
                            )
                        })
                        .collect::<Vec<_>>();

                    // Check if we were viewing comments for an article but that article is
                    // not in the update anymore.
                    let viewing_comments = views.iter().any(|article_entity| {
                        app.read_entity(article_entity, |article_view, _cx| {
                            article_view.viewing_comments
                        })
                    });

                    app.update_entity(&entity_content, |content, cx| {
                        if viewing_id.is_some() && !viewing_comments {
                            content.comment_entities.clear();
                        }

                        content.articles = views;
                        content.list_state.reset(content.articles.len());
                        content.article_ranks = current_ranking_map;
                        content.article_comment_counts = current_comment_counts;
                        content.background_refresh_count += 1;
                        cx.emit(ContentEvent::TotalArticles(content.articles.len()));
                        cx.emit(ContentEvent::TotalRefreshes(
                            content.background_refresh_count,
                        ));
                        cx.emit(ContentEvent::Error(None));
                        cx.notify();
                    });
                }
                Err(background_error) => {
                    error!("Received error from event source channel: {background_error:?}");
                    match background_error {
                        BackGroundError::Terminated(article_type) => {
                            app.update_entity(&entity_content, |_content_view, cx| {
                                cx.emit(ContentEvent::Terminated(article_type));
                                cx.notify();
                            })
                        }
                        BackGroundError::EventFailed(error) => {
                            app.update_entity(&entity_content, |_content_view, cx| {
                                cx.emit(ContentEvent::Error(Some(error)));
                                cx.notify();
                            })
                        }
                    };
                }
            }
        }
        log::warn!("Foreground events have terminated");
    })
    .detach();

    start_background_article_list_subscription(app, tx)
}

/// Starts a background subscription to the article list
/// and spawns a task to send articles to the foreground.
///
/// # Arguments
///
/// * `app` - A mutable reference to the application instance.
/// * `tx` - A sender channel for pushing article updates from background to foreground.
///
/// # Returns
///
/// Returns a `gpui::Task<()>` representing the spawned background task.
pub(crate) fn start_background_article_list_subscription(
    app: &mut App,
    mut tx: channel::mpsc::Sender<Result<Vec<Item>, BackGroundError>>,
) -> gpui::Task<()> {
    let ArticleSelection {
        viewing_article_type,
        viewing_article_total,
    } = app.read_global(|selection: &ArticleSelection, _app| *selection);

    info!("Starting background task for category {viewing_article_type} {viewing_article_total}");

    let client = app.read_global(|client: &ApiClientState, _app| client.0.clone());

    app.background_executor().spawn(Compat::new(async move {
        let (mut rx, handle) = subscribe_to_article_list(viewing_article_type);

        while let Some(event) = rx.recv().await {
            let article_ids = event
                .data
                .into_iter()
                .take(viewing_article_total)
                .collect::<Vec<_>>();

            let result = client
                .items(&article_ids)
                .try_collect::<Vec<_>>()
                .await
                .map_err(|err| {
                    BackGroundError::EventFailed(format!("Failed to fetch updated items: {err}"))
                });

            if let Err(err) = tx.send(result).await {
                error!("UI foreground send channel is closed: {err}");
                break;
            }
        }

        log::warn!("Background events have terminated");

        if let Err(err) = tx
            .send(Err(BackGroundError::Terminated(viewing_article_type)))
            .await
        {
            error!("Failed to send error {err}");
        }

        handle.abort();
    }))
}

const MARGIN: Pixels = px(10.0);

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
                    this.articles_width = (event.position.x - MARGIN).max(px(100.0));
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
                        cx.listener(|this, _event, _window, cx| {
                            this.is_dragging_divider = true;
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                        if this.is_dragging_divider {
                            this.articles_width = event.position.x;
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
                                div.child("Fetching comments...")
                            })
                            .when(
                                !self.comment_entities.is_empty() && !self.fetching_comments,
                                |div| self.render_comments(cx, theme, div),
                            ),
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
                            content_entity.update(app, |content, cx| {
                                content.comment_entities.clear();

                                for article_entity in &content.articles {
                                    article_entity.update(cx, |article_entity, _cx| {
                                        article_entity.viewing_comments = false;
                                    });
                                }
                            });
                        }),
                )
                .children(comment_entities.clone()),
        )
    }
}
