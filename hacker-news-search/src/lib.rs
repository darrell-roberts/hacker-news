use hacker_news_api::{ApiClient, Item};
use std::{future::Future, path::Path, pin::Pin};
use tantivy::{
    directory::{error::OpenDirectoryError, MmapDirectory},
    query::QueryParser,
    schema::{Schema, FAST, INDEXED, STORED, STRING, TEXT},
    Index, IndexReader, IndexWriter, Searcher, TantivyDocument, TantivyError,
};
use thiserror::Error;

pub const ITEM_ID: &str = "id";
pub const ITEM_PARENT_ID: &str = "parent_id";
pub const ITEM_TITLE: &str = "title";
pub const ITEM_BODY: &str = "body";
pub const ITEM_URL: &str = "url";
pub const ITEM_BY: &str = "by";
pub const ITEM_TYPE: &str = "type";
pub const ITEM_RANK: &str = "rank";
pub const ITEM_DESCENDANT_COUNT: &str = "descendants";
pub const ITEM_CATEGORY: &str = "category";

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
        let title = self.schema.get_field(ITEM_TITLE)?;
        let body = self.schema.get_field(ITEM_BODY)?;

        Ok(QueryParser::for_index(&self.index, vec![title, body]))
    }
}

fn article_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_u64_field(ITEM_RANK, STORED | INDEXED | FAST);
    schema_builder.add_u64_field(ITEM_ID, STORED | INDEXED | FAST);
    schema_builder.add_u64_field(ITEM_PARENT_ID, STORED | INDEXED | FAST);
    schema_builder.add_text_field(ITEM_TITLE, STORED | TEXT);
    schema_builder.add_text_field(ITEM_BODY, TEXT | STORED);
    schema_builder.add_text_field(ITEM_URL, STRING);
    schema_builder.add_text_field(ITEM_BY, STRING);
    schema_builder.add_text_field(ITEM_TYPE, TEXT | STORED);
    schema_builder.add_u64_field(ITEM_DESCENDANT_COUNT, STORED | INDEXED);
    schema_builder.add_text_field(ITEM_CATEGORY, TEXT | STORED);

    schema_builder.build()
}

pub fn index_articles<'a>(
    ctx: &'a SearchContext,
    client: &'a ApiClient,
    writer: &'a mut IndexWriter,
    articles: &'a [Item],
    category: &'a str,
) -> Pin<Box<impl Future<Output = Result<(), SearchError>> + use<'a>>> {
    Box::pin(async move {
        let id = ctx.schema.get_field(ITEM_ID)?;
        let parent = ctx.schema.get_field(ITEM_PARENT_ID)?;
        let title = ctx.schema.get_field(ITEM_TITLE)?;
        let body = ctx.schema.get_field(ITEM_BODY)?;
        let url = ctx.schema.get_field(ITEM_URL)?;
        let by = ctx.schema.get_field(ITEM_BY)?;
        let ty = ctx.schema.get_field(ITEM_TYPE)?;
        let rank = ctx.schema.get_field(ITEM_RANK)?;
        let descendant_count = ctx.schema.get_field(ITEM_DESCENDANT_COUNT)?;
        let category_field = ctx.schema.get_field(ITEM_CATEGORY)?;

        for (article, index) in articles.iter().zip(1..) {
            let mut doc = TantivyDocument::new();
            doc.add_u64(rank, index);
            doc.add_u64(id, article.id);
            if let Some(id) = article.parent {
                doc.add_u64(parent, id);
            }
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

            if !article.kids.is_empty() {
                let children = client.items(&article.kids).await?;
                index_articles(ctx, client, writer, &children, category).await?;
            }

            if let Some(n) = article.descendants {
                doc.add_u64(descendant_count, n);
            }

            if article.ty == "story" {
                doc.add_text(category_field, category);
            }

            writer.add_document(doc)?;
        }
        Ok(())
    })
}

pub async fn index(ctx: &SearchContext) -> Result<(), SearchError> {
    let client = ApiClient::new()?;

    let articles = client
        .articles(25, hacker_news_api::ArticleType::Top)
        .await?;

    let mut writer: IndexWriter = ctx.index.writer(50_000_000)?;
    index_articles(ctx, &client, &mut writer, &articles, "top").await?;
    writer.commit()?;

    Ok(())
}
