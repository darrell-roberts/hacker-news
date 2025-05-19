//! Search document storage and retrieval.
use hacker_news_api::ArticleType;
use log::info;
use std::{fs::create_dir_all, path::Path};
use tantivy::{
    directory::{error::OpenDirectoryError, MmapDirectory},
    query::{QueryParser, QueryParserError},
    schema::{
        Field, IndexRecordOption, Schema, TextFieldIndexing, TextOptions, FAST, INDEXED, STORED,
        STRING, TEXT,
    },
    Index, IndexReader, Searcher, TantivyError,
};
use thiserror::Error;

pub mod api;
pub mod create_index;

pub use create_index::*;

#[derive(Clone, Copy, Debug)]
pub struct HackerNewsFields {
    id: Field,
    parent_id: Field,
    title: Field,
    body: Field,
    url: Field,
    by: Field,
    ty: Field,
    rank: Field,
    descendant_count: Field,
    category: Field,
    time: Field,
    story_id: Field,
    kids: Field,
    score: Field,
}

/// The indexes for each category
pub struct HackerNewsIndices {
    top: Index,
    ask: Index,
    best: Index,
    job: Index,
    new: Index,
    show: Index,
}

impl HackerNewsIndices {
    /// Get the index for the article type.
    pub fn get_index(&self, article_type: ArticleType) -> &Index {
        match article_type {
            ArticleType::New => &self.new,
            ArticleType::Best => &self.best,
            ArticleType::Top => &self.top,
            ArticleType::Ask => &self.ask,
            ArticleType::Show => &self.show,
            ArticleType::Job => &self.job,
        }
    }
}

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
    indices: HackerNewsIndices,
    active_index: ArticleType,
    fields: HackerNewsFields,
}

fn create_indices(base_path: &Path, schema: &Schema) -> Result<HackerNewsIndices, SearchError> {
    let create_index = |article_type: ArticleType| -> Result<Index, SearchError> {
        let key = article_type.as_str();
        let full_path = base_path.join(key);
        if !full_path.exists() {
            info!("Creating directory {full_path:?} for index {key}");
            create_dir_all(full_path)?;
        }

        let index =
            Index::open_or_create(MmapDirectory::open(base_path.join(key))?, schema.clone())?;
        Ok(index)
    };

    Ok(HackerNewsIndices {
        top: create_index(ArticleType::Top)?,
        ask: create_index(ArticleType::Ask)?,
        best: create_index(ArticleType::Best)?,
        job: create_index(ArticleType::Job)?,
        new: create_index(ArticleType::New)?,
        show: create_index(ArticleType::Show)?,
    })
}

impl SearchContext {
    pub fn new(index_path: &Path, active_index: ArticleType) -> Result<Self, SearchError> {
        let (schema, fields) = document_schema();
        let indices = create_indices(index_path, &schema)?;
        let reader = indices.get_index(active_index).reader()?;

        Ok(SearchContext {
            reader,
            active_index,
            indices,
            schema,
            fields,
        })
    }

    pub fn activate_index(&mut self, active_index: ArticleType) -> Result<(), SearchError> {
        self.active_index = active_index;
        self.reader = self.indices.get_index(active_index).reader()?;
        Ok(())
    }

    pub fn searcher(&self) -> Searcher {
        self.reader.searcher()
    }

    pub fn query_parser(&self) -> QueryParser {
        let title = self.fields.title;
        let body = self.fields.body;

        QueryParser::for_index(self.indices.get_index(self.active_index), vec![title, body])
    }

    /// Get the active index category.
    pub fn active_category(&self) -> ArticleType {
        self.active_index
    }

    /// Get the total number of documents in the active index.
    pub fn doc_count(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    pub fn writer_context(&self) -> Result<WriteContext<'static>, SearchError> {
        let index = self.indices.get_index(self.active_index);
        WriteContext::new(
            self.fields,
            index.writer(50_000_000)?,
            self.active_index.as_str(),
        )
    }

    pub fn refresh_reader(&self) -> Result<(), SearchError> {
        Ok(self.reader.reload()?)
    }
}

fn document_schema() -> (Schema, HackerNewsFields) {
    let mut schema_builder = Schema::builder();

    let body_field_indexing = TextFieldIndexing::default()
        .set_tokenizer("en_stem")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let body_field_options = TextOptions::default()
        .set_indexing_options(body_field_indexing)
        .set_stored();

    let title_field_indexing =
        TextFieldIndexing::default().set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let title_field_options = TextOptions::default()
        .set_indexing_options(title_field_indexing)
        .set_stored();

    let fields = HackerNewsFields {
        id: schema_builder.add_u64_field(ITEM_ID, STORED | INDEXED | FAST),
        parent_id: schema_builder.add_u64_field(ITEM_PARENT_ID, STORED | INDEXED | FAST),
        title: schema_builder.add_text_field(ITEM_TITLE, title_field_options.clone()),
        body: schema_builder.add_text_field(ITEM_BODY, body_field_options),
        url: schema_builder.add_text_field(ITEM_URL, STRING | STORED),
        by: schema_builder.add_text_field(ITEM_BY, STRING | STORED),
        ty: schema_builder.add_text_field(ITEM_TYPE, TEXT | STORED),
        rank: schema_builder.add_u64_field(ITEM_RANK, STORED | INDEXED | FAST),
        descendant_count: schema_builder.add_u64_field(ITEM_DESCENDANT_COUNT, STORED | INDEXED),
        category: schema_builder.add_text_field(ITEM_CATEGORY, STRING),
        time: schema_builder.add_u64_field(ITEM_TIME, STORED | INDEXED | FAST),
        story_id: schema_builder.add_u64_field(ITEM_STORY_ID, FAST | INDEXED | STORED),
        kids: schema_builder.add_u64_field(ITEM_KIDS, FAST | INDEXED | STORED),
        score: schema_builder.add_u64_field(ITEM_SCORE, INDEXED | STORED),
    };

    (schema_builder.build(), fields)
}
