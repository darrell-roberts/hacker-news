use hacker_news_api::{ApiClient, Item};
use std::path::Path;
use tantivy::{
    directory::{error::OpenDirectoryError, MmapDirectory},
    query::QueryParser,
    schema::{Schema, INDEXED, STORED, STRING, TEXT},
    Index, IndexReader, IndexWriter, Searcher, TantivyDocument, TantivyError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Failed to open or create index")]
    OpenCreate(#[from] TantivyError),
    #[error("Failed to open directory")]
    OpenDirectory(#[from] OpenDirectoryError),
    #[error("API client error")]
    Client(#[from] anyhow::Error),
}

pub struct SearchContext {
    index: Index,
    reader: IndexReader,
    pub schema: Schema,
}

impl SearchContext {
    pub fn new(index_path: &Path) -> Result<Self, SearchError> {
        let schema = article_schema();
        let index = Index::open_or_create(MmapDirectory::open(index_path)?, schema.clone())?;
        let reader = index.reader()?;
        Ok(SearchContext {
            index,
            reader,
            schema,
        })
    }

    pub fn searcher(&self) -> Searcher {
        self.reader.searcher()
    }

    pub fn query(&self) -> Result<QueryParser, SearchError> {
        let title = self.schema.get_field("title")?;
        let body = self.schema.get_field("body")?;

        Ok(QueryParser::for_index(&self.index, vec![title, body]))
    }
}

fn article_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_u64_field("id", STORED | INDEXED);
    schema_builder.add_text_field("title", STORED | TEXT);
    schema_builder.add_text_field("body", TEXT | STORED);
    schema_builder.add_text_field("url", STRING);
    schema_builder.add_text_field("by", STRING);
    schema_builder.add_text_field("ty", TEXT | STORED);

    schema_builder.build()
}

pub fn index_articles(ctx: &SearchContext, articles: &[Item]) -> Result<(), SearchError> {
    let mut writer: IndexWriter = ctx.index.writer(50_000_000)?;

    let id = ctx.schema.get_field("id")?;
    let title = ctx.schema.get_field("title")?;
    let body = ctx.schema.get_field("body")?;
    let url = ctx.schema.get_field("url")?;
    let by = ctx.schema.get_field("by")?;
    let ty = ctx.schema.get_field("ty")?;

    for article in articles {
        let mut doc = TantivyDocument::new();
        doc.add_u64(id, article.id);
        if let Some(t) = article.title.as_deref() {
            doc.add_text(title, t);
        }
        if let Some(t) = article.text.as_deref() {
            doc.add_text(body, t);
        }
        if let Some(u) = article.url.as_deref() {
            doc.add_text(url, u);
        }
        doc.add_text(by, &article.by);
        doc.add_text(ty, &article.ty);

        writer.add_document(doc)?;
    }

    writer.commit()?;

    Ok(())
}

pub async fn index(ctx: &SearchContext) -> Result<(), SearchError> {
    let client = ApiClient::new()?;

    let articles = client
        .articles(50, hacker_news_api::ArticleType::Top)
        .await?;

    index_articles(ctx, &articles)?;

    Ok(())
}
