use hacker_news_search::{index, SearchContext};
use std::{fs::exists, path::Path};
use tokio::fs::{create_dir_all, remove_dir_all};

const INDEX_PATH: &str = "/tmp/hacker-news";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // create_index().await?;
    // top_stories()?;
    comments()?;
    Ok(())
}

async fn create_index() -> anyhow::Result<()> {
    if exists(INDEX_PATH)? {
        remove_dir_all(INDEX_PATH).await?;
    }
    create_dir_all("/tmp/hacker-news").await?;

    let ctx = SearchContext::new(Path::new(INDEX_PATH))?;
    index(&ctx).await?;
    Ok(())
}

fn comments() -> anyhow::Result<()> {
    let ctx = SearchContext::new(Path::new(INDEX_PATH))?;
    dbg!(ctx.comments(42344002, 0)?);
    Ok(())
}

fn top_stories() -> anyhow::Result<()> {
    let ctx = SearchContext::new(Path::new(INDEX_PATH))?;
    dbg!(ctx.top_stories(0)?);

    Ok(())
}
