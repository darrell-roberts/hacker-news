use hacker_news_search::{index, SearchContext};
use std::path::Path;
use tantivy::{collector::TopDocs, Document, TantivyDocument};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ctx = SearchContext::new(Path::new("/tmp/hacker-news"))?;
    // index(&ctx).await?;
    search(&ctx)?;
    Ok(())
}

async fn create_index(ctx: &SearchContext) -> anyhow::Result<()> {
    Ok(index(ctx).await?)
}

fn search(ctx: &SearchContext) -> anyhow::Result<()> {
    let searcher = ctx.searcher();

    let query = ctx.query()?.parse_query("evil")?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    for (_score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        println!("{}", retrieved_doc.to_json(&ctx.schema));
    }

    Ok(())
}
