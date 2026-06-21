//! Common functions.
use crate::{
    ApiClientState, CONFIG_FILE, Config, UrlHover, article::ArticleView, comment::CommentView,
    content::ContentEvent, theme::Theme,
};
use futures::{StreamExt as _, TryStreamExt as _};
use gpui::{
    App, AppContext, AsyncApp, Entity, Hsla, Image, SharedString, StyleRefinement, Styled as _,
    http_client::Url,
};
use gpui_tokio::Tokio;
use log::error;
use std::{
    borrow::Cow,
    sync::{Arc, LazyLock},
};

/// An embedded SVG comment image.
pub static COMMENT_IMAGE: LazyLock<Arc<Image>> = LazyLock::new(|| {
    Arc::new(Image::from_bytes(
        gpui::ImageFormat::Svg,
        include_bytes!("../assets/comment.svg").into(),
    ))
});

/// Create comment entities by fetching the remote comments and
/// creating a comment entity for each.
///
/// # Arguments
///
/// * `app` - A mutable reference to the asynchronous application.
/// * `article_entity` - The entity representing the article to which the comments belong.
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
    let content_entity = app.read_entity(&article_entity, |article_view, _cx| {
        article_view.content_entity.clone()
    });
    app.update_entity(&content_entity, |_content_view, cx| {
        cx.emit(ContentEvent::Error(None));
    });
    let client = app.read_global(|client: &ApiClientState, _| client.0.clone());

    let item_stream = client
        .items(comment_ids)
        .into_stream()
        .filter_map(|comment_result| async move {
            match comment_result {
                Ok(comment) => Some(comment),
                Err(err) => {
                    error!("Failed to fetch comment: {err}");
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    let comment_items = Tokio::spawn(app, item_stream)
        .await
        .inspect_err(|err| error!("Failed to spawn task to fetch comments: {err}"))
        .unwrap_or_default();

    comment_items
        .into_iter()
        .map(|comment| CommentView::new(app, comment, article_entity.clone()))
        .collect()
}

/// Render the url with a unicode host if the url is using puny code.
pub fn url_punycode(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|parsed_url| {
            let host = parsed_url.host_str()?;
            let (host, result) = idna::domain_to_unicode(host);
            result.ok()?;

            Some(format!(
                "{}://{host}{}{}",
                parsed_url.scheme(),
                port_string(&parsed_url),
                parsed_url.path()
            ))
        })
        .unwrap_or_else(|| url.to_string())
}

/// Produce a port url part if the scheme and port combination are not standard.
fn port_string(parsed_url: &Url) -> Cow<'_, str> {
    let port: Cow<'_, str> = match parsed_url.port() {
        Some(80) if parsed_url.scheme() == "http" => "".into(),
        Some(443) if parsed_url.scheme() == "https" => "".into(),
        Some(port) => format!(":{port}").into(),
        None => "".into(),
    };
    port
}

pub fn hover_element(theme: Theme) -> impl Fn(StyleRefinement) -> StyleRefinement {
    move |style| style.text_color(brighten(theme.text_color().into(), 0.2))
}

pub fn save_config(config: Config) -> impl Future<Output = Result<(), anyhow::Error>> {
    hacker_news_config::save_config(config, CONFIG_FILE)
}

pub fn update_url(app: &mut App, url: Option<SharedString>) {
    let current_url = app.global::<UrlHover>();

    if url != current_url.0 {
        app.set_global(UrlHover(url));
    }
}

/// Brighten color
pub fn brighten(color: Hsla, amount: f32) -> Hsla {
    Hsla {
        l: (color.l + amount).clamp(0.0, 1.0),
        ..color
    }
}
