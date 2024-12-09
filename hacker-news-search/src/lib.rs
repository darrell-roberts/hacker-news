use std::path::Path;
use tantivy::{
    directory::{error::OpenDirectoryError, MmapDirectory},
    query::{QueryParser, QueryParserError},
    schema::{
        IndexRecordOption, Schema, TextFieldIndexing, TextOptions, FAST, INDEXED, STORED, STRING,
        TEXT,
    },
    Index, IndexReader, Searcher, TantivyError,
};
use thiserror::Error;

pub mod api;
pub mod create_index;

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
pub const ITEM_STORY_ID: &str = "story_id";
pub const ITEM_KIDS: &str = "kids";
pub const ITEM_SCORE: &str = "score";

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Tantivy error: {0}")]
    Tantivy(#[from] TantivyError),
    #[error("Failed to open directory: {0}")]
    OpenDirectory(#[from] OpenDirectoryError),
    #[error("API client error: {0}")]
    Client(#[from] anyhow::Error),
    #[error("Bad query: {0}")]
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

    let text_field_indexing = TextFieldIndexing::default()
        .set_tokenizer("en_stem")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_field_options = TextOptions::default()
        .set_indexing_options(text_field_indexing)
        .set_stored();

    schema_builder.add_u64_field(ITEM_RANK, FAST);
    schema_builder.add_u64_field(ITEM_ID, STORED | INDEXED | FAST);
    schema_builder.add_u64_field(ITEM_PARENT_ID, STORED | INDEXED | FAST);
    schema_builder.add_text_field(ITEM_TITLE, text_field_options.clone());
    schema_builder.add_text_field(ITEM_BODY, text_field_options);
    schema_builder.add_text_field(ITEM_URL, STRING | STORED);
    schema_builder.add_text_field(ITEM_BY, STRING | STORED);
    schema_builder.add_text_field(ITEM_TYPE, TEXT | STORED);
    schema_builder.add_u64_field(ITEM_DESCENDANT_COUNT, STORED | INDEXED);
    schema_builder.add_text_field(ITEM_CATEGORY, STRING);
    // schema_builder.add_date_field(ITEM_TIME, STORED | INDEXED | FAST);
    schema_builder.add_u64_field(ITEM_TIME, STORED | INDEXED | FAST);
    schema_builder.add_u64_field(ITEM_STORY_ID, FAST | INDEXED);
    schema_builder.add_u64_field(ITEM_KIDS, FAST | INDEXED | STORED);
    schema_builder.add_u64_field(ITEM_SCORE, INDEXED | STORED);

    schema_builder.build()
}
