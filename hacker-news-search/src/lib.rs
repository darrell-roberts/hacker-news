use hacker_news_api::ArticleType;
use log::info;
use std::{collections::HashMap, fs::create_dir_all, path::Path};
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
    #[error("Failed to create index folder")]
    IO(#[from] std::io::Error),
    #[error("Failed to transform doc")]
    BadDoc,
    #[error("Doc missing expected field {0}")]
    MissingField(String),
    #[error("Api timed out {0}")]
    TimedOut(String),
    #[error("Document does not exist")]
    MissingDoc,
    #[error("Failed to join async task: {0}")]
    Join(#[from] tokio::task::JoinError),
}

pub struct SearchContext {
    reader: IndexReader,
    schema: Schema,
    indices: HashMap<&'static str, Index>,
    active_index: ArticleType,
}

fn indices_map(
    base_path: &Path,
    schema: &Schema,
) -> Result<HashMap<&'static str, Index>, SearchError> {
    let keys = [
        ArticleType::Top.as_str(),
        ArticleType::Ask.as_str(),
        ArticleType::Best.as_str(),
        ArticleType::Job.as_str(),
        ArticleType::New.as_str(),
        ArticleType::Show.as_str(),
    ];

    let mut map = HashMap::new();

    for key in keys {
        let full_path = base_path.join(key);
        if !full_path.exists() {
            info!("Creating directory {full_path:?} for index {key}");
            create_dir_all(full_path)?;
        }

        let index =
            Index::open_or_create(MmapDirectory::open(base_path.join(key))?, schema.clone())?;
        map.insert(key, index);
    }

    Ok(map)
}

impl SearchContext {
    pub fn new(index_path: &Path, active_index: ArticleType) -> Result<Self, SearchError> {
        let schema = document_schema();
        let indices = indices_map(index_path, &schema)?;
        let reader = indices.get(active_index.as_str()).unwrap().reader()?;

        Ok(SearchContext {
            reader,
            active_index,
            indices,
            schema,
        })
    }

    pub fn activate_index(&mut self, active_index: ArticleType) -> Result<(), SearchError> {
        self.active_index = active_index;
        self.reader = self.indices.get(&active_index.as_str()).unwrap().reader()?;
        Ok(())
    }

    pub fn searcher(&self) -> Searcher {
        self.reader.searcher()
    }

    pub fn query_parser(&self) -> Result<QueryParser, SearchError> {
        let title = self.schema.get_field(ITEM_TITLE)?;
        let body = self.schema.get_field(ITEM_BODY)?;

        Ok(QueryParser::for_index(
            self.indices.get(self.active_index.as_str()).unwrap(),
            vec![title, body],
        ))
    }

    pub fn active_category(&self) -> ArticleType {
        self.active_index
    }
}

fn document_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    let text_field_indexing = TextFieldIndexing::default()
        .set_tokenizer("en_stem")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_field_options = TextOptions::default()
        .set_indexing_options(text_field_indexing)
        .set_stored();

    schema_builder.add_u64_field(ITEM_RANK, STORED | INDEXED | FAST);
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
    schema_builder.add_u64_field(ITEM_STORY_ID, FAST | INDEXED | STORED);
    schema_builder.add_u64_field(ITEM_KIDS, FAST | INDEXED | STORED);
    schema_builder.add_u64_field(ITEM_SCORE, INDEXED | STORED);

    schema_builder.build()
}
