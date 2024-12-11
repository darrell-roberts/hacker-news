//! Search API for top stories.
use super::Story;
use crate::{SearchContext, SearchError, ITEM_CATEGORY, ITEM_RANK, ITEM_TITLE, ITEM_TYPE};
use tantivy::{
    collector::TopDocs,
    query::{BooleanQuery, FuzzyTermQuery, Occur, TermQuery},
    schema::IndexRecordOption,
    Order, TantivyDocument, Term,
};

impl SearchContext {
    /// Lookup top stories applying limit and  offset pagination.
    pub fn top_stories(&self, limit: usize, offset: usize) -> Result<Vec<Story>, SearchError> {
        let query = self
            .query_parser()?
            .parse_query("category:top AND type:story")?;
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
        let type_story_query = Box::new(TermQuery::new(
            Term::from_field_text(self.schema.get_field(ITEM_TYPE)?, "story"),
            IndexRecordOption::Basic,
        ));

        let type_job_query = Box::new(TermQuery::new(
            Term::from_field_text(self.schema.get_field(ITEM_TYPE)?, "job"),
            IndexRecordOption::Basic,
        ));

        let category_query = Box::new(TermQuery::new(
            Term::from_field_text(self.schema.get_field(ITEM_CATEGORY)?, "top"),
            IndexRecordOption::Basic,
        ));

        let fuzzy_query = Box::new(FuzzyTermQuery::new(
            Term::from_field_text(self.schema.get_field(ITEM_TITLE)?, search),
            1,
            true,
        ));

        let query = BooleanQuery::new(vec![
            (
                Occur::Must,
                Box::new(BooleanQuery::new(vec![
                    (Occur::Should, type_story_query),
                    (Occur::Should, type_job_query),
                ])),
            ),
            (Occur::Must, category_query),
            (Occur::Must, fuzzy_query),
        ]);

        let searcher = self.searcher();
        let top_docs = TopDocs::with_limit(75)
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
}
