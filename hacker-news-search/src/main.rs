use hacker_news_search::{index, SearchContext, ITEM_PARENT_ID, ITEM_RANK};
use std::{fs::exists, path::Path};
use tantivy::{
    collector::TopDocs,
    query::{BooleanQuery, ExistsQuery, Occur, QueryClone},
    Document, Order, TantivyDocument, Term,
};
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

    // let query = ctx.query()?.parse_query("evil")?;
    // let query = BooleanQuery::new(vec![(
    //     Occur::MustNot,
    //     ExistsQuery::new_exists_query(ITEM_PARENT_ID.into()).box_clone(),
    // )]);

    // let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
    let top_docs = TopDocs::with_limit(25).order_by_u64_field(ITEM_RANK, Order::Asc);
    let docs = searcher.search(&query, &top_docs)?;

    for (_score, doc_address) in docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        println!("{}", retrieved_doc.to_json(&ctx.schema));
    }

    Ok(())
}
