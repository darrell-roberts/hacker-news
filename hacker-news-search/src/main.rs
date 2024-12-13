#![expect(dead_code)]
use hacker_news_api::ArticleType;
use hacker_news_search::{rebuild_index, SearchContext};
use std::{
    fs::exists,
    path::Path,
    sync::{Arc, RwLock},
};
use tokio::fs::{create_dir_all, remove_dir_all};

// const INDEX_PATH: &str = "/home/droberts/.local/share/Hacker News/hacker-news-index/";
const INDEX_PATH: &str = "/tmp/hacker-news";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    create().await?;
    top_stories()?;
    // comments()?;
    Ok(())
}

async fn create() -> anyhow::Result<()> {
    if exists(INDEX_PATH)? {
        remove_dir_all(INDEX_PATH).await?;
    }
    create_dir_all(INDEX_PATH).await?;

    let ctx = Arc::new(RwLock::new(SearchContext::new(
        Path::new(INDEX_PATH),
        ArticleType::Top,
    )?));
    rebuild_index(ctx.clone(), ArticleType::Top).await?;
    Ok(())
}

fn comments() -> anyhow::Result<()> {
    let ctx = SearchContext::new(Path::new(INDEX_PATH), ArticleType::Top)?;
    dbg!(ctx.comments(42344002, 10, 0)?);
    Ok(())
}

fn top_stories() -> anyhow::Result<()> {
    let ctx = SearchContext::new(Path::new(INDEX_PATH), ArticleType::Top)?;
    dbg!(ctx.top_stories(100, 0)?);

    Ok(())
}
