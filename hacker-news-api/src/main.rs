use anyhow::Error;
use hacker_news_api::ApiClient;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let top_stories = ApiClient::new()?.top_stories(20).await?;

    for story in top_stories {
        println!(
            "{} url: {}, comments: {}",
            story.title.unwrap_or_default(),
            story.url.unwrap_or_default(),
            story.kids.len()
        );
    }

    Ok(())
}
