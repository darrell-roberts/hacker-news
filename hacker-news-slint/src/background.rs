//! Background subscription to update article list.
use anyhow::Context;
use futures::TryStreamExt as _;
use hacker_news_api::{ApiClient, ArticleType, subscribe_to_article_list};
use log::{debug, error, info};
use slint::{ModelRc, VecModel, Weak};
use tokio::runtime::{self};

use crate::{AppWindow, ArticleData};

/// Start a tokio runtime and subscribe to the hacker news article
/// list event source. On each event fetch the stories and update
/// the model.
pub(crate) fn subscribe_to_stories(app_window: Weak<AppWindow>) {
    std::thread::Builder::new()
        .name("Background events".into())
        .spawn(move || {
            info!("Running in new background thread");
            let runtime = runtime::Builder::new_multi_thread()
                .name("Background updater")
                .worker_threads(5)
                .enable_all()
                .build()
                .unwrap();

            runtime.block_on(async move {
                info!("Running in tokio");

                let client = ApiClient::new().expect("Could not start api client");
                let app_window = app_window.clone();

                let (mut receiver, _handle) = subscribe_to_article_list(ArticleType::Top, 5);

                while let Some(event) = receiver.recv().await {
                    if let Err(err) = fetch_and_update(&client, &app_window, &event.data).await {
                        error!("Failed to fetch and update articles: {err}");
                    }
                }
            });
        })
        .expect("Could not start background thread");
}

async fn fetch_and_update(
    client: &ApiClient,
    app_window: &Weak<AppWindow>,
    ids: &[u64],
) -> anyhow::Result<()> {
    info!("Fetching articles");
    let items = client
        .items(ids)
        .try_collect::<Vec<_>>()
        .await
        .context("Failed to fetch articles")?;

    debug!("Updating model with {} items", items.len());
    app_window
        .upgrade_in_event_loop(move |app_window| {
            app_window.set_articles(ModelRc::new(
                items
                    .into_iter()
                    .map(|item| ArticleData {
                        author: item.by.into(),
                        title: item.title.unwrap_or_default().into(),
                        age: friendly_duration::parse_friendly_age(item.time)
                            .unwrap_or_default()
                            .into(),
                    })
                    .collect::<VecModel<_>>(),
            ));
        })
        .context("Failed to update model")?;

    Ok(())
}
