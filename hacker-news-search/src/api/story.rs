//! Search API for top stories.
use super::Story;
use crate::{SearchContext, SearchError, ITEM_RANK};
use std::sync::OnceLock;
use tantivy::{
    collector::TopDocs,
    query::{Query, TermQuery},
    schema::IndexRecordOption,
    Order, TantivyDocument, Term,
};

static TOP_STORIES_QUERY: OnceLock<Box<dyn Query>> = OnceLock::new();

impl SearchContext {
    /// Lookup top stories applying limit and  offset pagination.
    pub fn top_stories(&self, limit: usize, offset: usize) -> Result<Vec<Story>, SearchError> {
        // TODO: Remove unwrap when https://doc.rust-lang.org/std/sync/struct.OnceLock.html#method.get_or_try_init stabilizes
        let query = TOP_STORIES_QUERY.get_or_init(|| {
            self.query_parser()
                .parse_query("type: IN [story, job, poll]")
                .unwrap()
        });
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
            .map(|id| -> Box<dyn Query> {
                Box::new(TermQuery::new(
                    Term::from_field_u64(self.fields.id, id),
                    IndexRecordOption::Basic,
                ))
            })
            .ok();

        let query = {
            match story_id_query {
                Some(q) => q,
                None => self
                    .query_parser()
                    .parse_query(&format!("type: IN [story, job, poll] AND title:{search}"))?,
            }
        };

        let searcher = self.searcher();
        let top_docs = TopDocs::with_limit(limit).and_offset(offset);

        searcher
            .search(&query, &top_docs)?
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
