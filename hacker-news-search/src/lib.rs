use std::path::Path;
use tantivy::{
    directory::{error::OpenDirectoryError, MmapDirectory},
    query::{QueryParser, QueryParserError},
    schema::{Schema, FAST, INDEXED, STORED, STRING, TEXT},
    Index, IndexReader, Searcher, TantivyError,
};
use thiserror::Error;

mod create_index;
mod stories;

pub use create_index::*;

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
pub const ITEM_TIME: &str = "time";

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Failed to open or create index")]
    OpenCreate(#[from] TantivyError),
    #[error("Failed to open directory")]
    OpenDirectory(#[from] OpenDirectoryError),
    #[error("API client error")]
    Client(#[from] anyhow::Error),
    #[error("Bad query")]
    Query(#[from] QueryParserError),
}

pub struct SearchContext {
    index: Index,
    reader: IndexReader,
    schema: Schema,
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

    pub fn query_parser(&self) -> Result<QueryParser, SearchError> {
        let title = self.schema.get_field(ITEM_TITLE)?;
        let body = self.schema.get_field(ITEM_BODY)?;

        Ok(QueryParser::for_index(&self.index, vec![title, body]))
    }
}

fn article_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_u64_field(ITEM_RANK, FAST);
    schema_builder.add_u64_field(ITEM_ID, STORED | INDEXED | FAST);
    schema_builder.add_u64_field(ITEM_PARENT_ID, STORED | INDEXED | FAST);
    schema_builder.add_text_field(ITEM_TITLE, STORED | TEXT);
    schema_builder.add_text_field(ITEM_BODY, TEXT | STORED);
    schema_builder.add_text_field(ITEM_URL, STRING | STORED);
    schema_builder.add_text_field(ITEM_BY, STRING | STORED);
    schema_builder.add_text_field(ITEM_TYPE, TEXT | STORED);
    schema_builder.add_u64_field(ITEM_DESCENDANT_COUNT, STORED | INDEXED);
    schema_builder.add_text_field(ITEM_CATEGORY, STRING);
    schema_builder.add_u64_field(ITEM_TIME, STORED | INDEXED | FAST);

    schema_builder.build()
}
