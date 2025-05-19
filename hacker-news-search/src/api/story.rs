//! Search API for top stories.
use super::Story;
use crate::{SearchContext, SearchError, ITEM_RANK};
use std::sync::OnceLock;
use tantivy::{
    collector::TopDocs,
    query::{BooleanQuery, Occur, Query, TermQuery},
    schema::{Field, IndexRecordOption},
    Order, TantivyDocument, Term,
};

static STORY_OR_JOB_OR_POLL: OnceLock<BooleanQuery> = OnceLock::new();

fn story_job_poll(type_field: Field) -> BooleanQuery {
    let mk_query = |ty: &str| -> (Occur, Box<dyn Query>) {
        (
            Occur::Should,
            Box::new(TermQuery::new(
                Term::from_field_text(type_field, ty),
                IndexRecordOption::Basic,
            )),
        )
    };
    BooleanQuery::new([mk_query("story"), mk_query("job"), mk_query("poll")].into())
}

impl SearchContext {
    /// Lookup top stories applying limit and  offset pagination.
    pub fn top_stories(&self, limit: usize, offset: usize) -> Result<Vec<Story>, SearchError> {
        let query = STORY_OR_JOB_OR_POLL.get_or_init(|| story_job_poll(self.fields.ty));
        let searcher = self.searcher();
        let top_docs = TopDocs::with_limit(limit)
            // Pagination
            .and_offset(offset)
            // Ordering
            .order_by_u64_field(ITEM_RANK, Order::Asc);

        searcher
            .search(query, &top_docs)?
            .into_iter()
            .map(|(_, doc_address)| self.to_story(searcher.doc(doc_address)?))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Search all stories with term and offset pagination.
    pub fn search_stories(
        &self,
        search: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Story>, SearchError> {
        let story_id_query = search
            .parse::<u64>()
            .map(|id| {
                TermQuery::new(
                    Term::from_field_u64(self.fields.id, id),
                    IndexRecordOption::Basic,
                )
            })
            .ok();

        let query: &dyn Query = {
            match story_id_query.as_ref() {
                Some(q) => q,
                None => &BooleanQuery::new(vec![
                    (
                        Occur::Must,
                        Box::new(TermQuery::new(
                            Term::from_field_text(self.fields.title, &search.to_lowercase()),
                            IndexRecordOption::WithFreqs,
                        )),
                    ),
                    (Occur::Must, Box::new(story_job_poll(self.fields.ty))),
                ]),
            }
        };

        let searcher = self.searcher();
        let top_docs = TopDocs::with_limit(limit).and_offset(offset);

        searcher
            .search(query, &top_docs)?
            .into_iter()
            .map(|(_, doc_address)| self.to_story(searcher.doc(doc_address)?))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Lookup a single story.
    pub fn story(&self, story_id: u64) -> Result<Story, SearchError> {
        self.to_story(self.story_doc(story_id)?)
    }

    pub fn story_doc(&self, story_id: u64) -> Result<TantivyDocument, SearchError> {
        let searcher = self.searcher();
        let top_docs = TopDocs::with_limit(1);
        let story_query: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_u64(self.fields.id, story_id),
            IndexRecordOption::Basic,
        ));

        let (_score, doc_address) = searcher
            .search(&story_query, &top_docs)?
            .into_iter()
            .next()
            .ok_or_else(|| SearchError::MissingDoc)?;

        Ok(searcher.doc(doc_address)?)
    }
}
