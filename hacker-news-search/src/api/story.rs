//! Search API for top stories.
use super::Story;
use crate::{SearchContext, SearchError, ITEM_ID, ITEM_RANK, ITEM_TITLE};
use anyhow::Context;
use tantivy::{
    collector::TopDocs,
    query::{FuzzyTermQuery, Query, TermQuery},
    schema::IndexRecordOption,
    Order, TantivyDocument, Term,
};

impl SearchContext {
    /// Lookup top stories applying limit and  offset pagination.
    pub fn top_stories(&self, limit: usize, offset: usize) -> Result<Vec<Story>, SearchError> {
        let query = self
            .query_parser()?
            .parse_query("type: IN [story, job, poll]")?;
        let searcher = self.searcher();

        let top_docs = TopDocs::with_limit(limit)
            // Pagination
            .and_offset(offset)
            // Ordering
            .order_by_u64_field(ITEM_RANK, Order::Asc);

        let stories = searcher
            .search(&query, &top_docs)?
            .into_iter()
            .map(|(_, doc_address)| {
                let doc = searcher.doc::<TantivyDocument>(doc_address)?;
                self.to_story(doc)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(stories)
    }

    /// Search all stories with term and offset pagination.
    pub fn search_stories(&self, search: &str, offset: usize) -> Result<Vec<Story>, SearchError> {
        let fuzzy_query: Box<dyn Query> = Box::new(FuzzyTermQuery::new(
            Term::from_field_text(self.schema.get_field(ITEM_TITLE)?, search),
            1,
            true,
        ));

        // let term_query: Box<dyn Query> = Box::new(TermQuery::new(
        //     Term::from_field_text(self.schema.get_field(ITEM_TITLE)?, search),
        //     IndexRecordOption::Basic,
        // ));

        let story_id_query = search
            .parse::<u64>()
            .context("Not an id")
            .and_then(|id| {
                self.schema
                    .get_field(ITEM_ID)
                    .map(|field| (id, field))
                    .context("No field")
            })
            .map(|(id, field)| -> Box<dyn Query> {
                Box::new(TermQuery::new(
                    Term::from_field_u64(field, id),
                    IndexRecordOption::Basic,
                ))
            })
            .ok();

        let query: Box<dyn Query> = story_id_query.unwrap_or(fuzzy_query);

        let searcher = self.searcher();
        let top_docs = TopDocs::with_limit(75).and_offset(offset);

        let stories = searcher
            .search(&query, &top_docs)?
            .into_iter()
            .map(|(_, doc_address)| {
                let doc = searcher.doc::<TantivyDocument>(doc_address)?;
                self.to_story(doc)
            })
            .collect::<Result<Vec<_>, _>>()?;

        dbg!(&stories);
        Ok(stories)
    }

    /// Lookup a single story.
    pub fn story(&self, story_id: u64) -> Result<Story, SearchError> {
        self.to_story(self.story_doc(story_id)?)
    }

    pub fn story_doc(&self, story_id: u64) -> Result<TantivyDocument, SearchError> {
        let searcher = self.searcher();
        let top_docs = TopDocs::with_limit(1);
        let story_query: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_u64(self.schema.get_field(ITEM_ID)?, story_id),
            IndexRecordOption::Basic,
        ));

        let (_score, doc_address) = searcher
            .search(&story_query, &top_docs)?
            .into_iter()
            .next()
            .ok_or_else(|| SearchError::MissingDoc)?;

        Ok(searcher.doc::<TantivyDocument>(doc_address)?)
    }
}
