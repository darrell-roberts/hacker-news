use anyhow::Error;
use hacker_news_api::{subscribe_top_stories, ApiClient};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    // let top_stories = ApiClient::new()?.top_stories(20).await?;

    // let top_stories = ApiClient::new()?.top_stories_stream().await?;

    // for story in top_stories {
    //     println!(
    //         "{} url: {}, comments: {}",
    //         story.title.unwrap_or_default(),
    //         story.url.unwrap_or_default(),
    //         story.kids.len()
    //     );
    // }

    let mut rx = subscribe_top_stories();

    let mut old_keys = Vec::new();

    while let Some(mut data) = rx.recv().await {
        data.data.sort_unstable();
        // println!("received data: {data:?}");

        let new_keys = data
            .data
            .iter()
            .copied()
            .take(50)
            .filter(|id| old_keys.binary_search(id).is_err())
            .collect::<Vec<_>>();

        println!("new keys total {} {new_keys:?}", new_keys.len());

        old_keys = data.data;
    }

    Ok(())
}
