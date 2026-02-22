//! Common functions.
//!

use chrono::{DateTime, Utc};
use futures::TryStreamExt as _;
use gpui::{AsyncApp, Entity, Image};
use std::sync::{Arc, LazyLock};

use crate::{article::ArticleView, comment::CommentView, ApiClientState};

/// An embedded SVG comment image.
pub static COMMENT_IMAGE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        include_bytes!("../assets/comment.svg").into(),
    ))
});

/// Extract the duration from a UNIX time and convert duration into a human
/// friendly sentence.
///
/// # Arguments
///
/// * `time` - A UNIX timestamp as a `u64`.
///
/// # Returns
///
/// An `Option<String>` containing a human-friendly sentence representing the duration
/// since the given UNIX time, or `None` if the timestamp is invalid.
pub fn parse_date(time: u64) -> Option<String> {
    let duration =
        DateTime::<Utc>::from_timestamp(time.try_into().ok()?, 0).map(|then| Utc::now() - then)?;

    let hours = duration.num_hours();
    let minutes = duration.num_minutes();
    let days = duration.num_days();

    match (days, hours, minutes) {
        (0, 0, 1) => "1 minute ago".to_string(),
        (0, 0, m) => format!("{m} minutes ago"),
        (0, 1, _) => "1 hour ago".to_string(),
        (0, h, _) => format!("{h} hours ago"),
        (1, _, _) => "1 day ago".to_string(),
        (d, _, _) => format!("{d} days ago"),
    }
    .into()
}

/// Create comment entities by fetching the remote comments and
/// creating a comment entity for each.
///
/// # Arguments
///
/// * `app` - A mutable reference to the asynchronous application.
/// * `article_entity` - The entity representing the article to which the comments belong.
/// * `comment_ids` - A slice of comment IDs to fetch and create entities for.
///
/// # Returns
///
/// A vector of `Entity<CommentView>` representing the created comment entities.
///
pub async fn comment_entities(
    app: &mut AsyncApp,
    article_entity: Entity<ArticleView>,
    comment_ids: &[u64],
) -> Vec<Entity<CommentView>> {
    let client = app.read_global(|client: &ApiClientState, _| client.0.clone());
    let comment_items =
        async_compat::Compat::new(client.items(comment_ids).try_collect::<Vec<_>>())
            .await
            .unwrap_or_default();

    comment_items
        .into_iter()
        .map(|comment| CommentView::new(app, comment, article_entity.clone()))
        .collect()
}
