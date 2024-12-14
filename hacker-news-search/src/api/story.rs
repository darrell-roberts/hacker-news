//! Search API for top stories.
use super::Story;
use crate::{SearchContext, SearchError, ITEM_ID, ITEM_RANK, ITEM_TITLE, ITEM_TYPE};
use anyhow::Context;
use tantivy::{
    collector::TopDocs,
    query::{BooleanQuery, FuzzyTermQuery, Occur, Query, TermQuery},
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
        // let type_story_query: Box<dyn Query> = Box::new(TermQuery::new(
        //     Term::from_field_text(self.schema.get_field(ITEM_TYPE)?, "story"),
        //     IndexRecordOption::Basic,
        // ));

        let type_job_query: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_text(self.schema.get_field(ITEM_TYPE)?, "job"),
            IndexRecordOption::Basic,
        ));

        let fuzzy_query: Box<dyn Query> = Box::new(FuzzyTermQuery::new(
            Term::from_field_text(self.schema.get_field(ITEM_TITLE)?, search),
            1,
            true,
        ));

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

        let options = [
            Some((Occur::Should, fuzzy_query)),
            // Some((Occur::Should, type_story_query)),
            Some((Occur::Should, type_job_query)),
            story_id_query.map(|query| (Occur::Should, query)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        // let query = BooleanQuery::new(vec![
        //     (Occur::Must, Box::new(BooleanQuery::new(options))),
        //     (Occur::Must, fuzzy_query),
        // ]);
        let query = BooleanQuery::new(options);

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

        Ok(stories)
    }

    pub fn story(&self, story_id: u64) -> Result<Story, SearchError> {
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

        let doc = searcher.doc::<TantivyDocument>(doc_address)?;
        self.to_story(doc)
    }
}
