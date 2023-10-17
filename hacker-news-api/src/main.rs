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

    while let Some(data) = rx.recv().await {
        println!("received data: {data:?}");
    }

    Ok(())
}
