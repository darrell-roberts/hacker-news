use hacker_news_search::{index, SearchContext, ITEM_RANK};
use std::{fs::exists, path::Path};
use tantivy::{collector::TopDocs, Document, Order, TantivyDocument};
use tokio::fs::{create_dir_all, remove_dir_all};

const INDEX_PATH: &str = "/tmp/hacker-news";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    search()?;
    // create_index().await?;
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

fn search() -> anyhow::Result<()> {
    let ctx = SearchContext::new(Path::new(INDEX_PATH))?;
    let searcher = ctx.searcher();

    let query = ctx.query()?.parse_query("category:top AND type:story")?;

    let top_docs = TopDocs::with_limit(25).order_by_u64_field(ITEM_RANK, Order::Asc);
    let docs = searcher.search(&query, &top_docs)?;

    for (_score, doc_address) in docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        println!("{}", retrieved_doc.to_json(&ctx.schema));
    }

    Ok(())
}
