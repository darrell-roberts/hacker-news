//! Search API for user comments.
use super::{Comment, Story};
use crate::{SearchContext, SearchError, ITEM_BODY, ITEM_ID, ITEM_RANK, ITEM_STORY_ID, ITEM_TYPE};
use tantivy::{
    collector::{Count, MultiCollector, TopDocs},
    query::{BooleanQuery, FuzzyTermQuery, Occur, Query, TermQuery},
    schema::IndexRecordOption,
    Order, Searcher, TantivyDocument, Term,
};

impl SearchContext {
    /// Lookup comments by parent_id with limit pagination offset.
    pub fn comments(
        &self,
        parent_id: u64,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Comment>, usize), SearchError> {
        let query = self
            .query_parser()?
            .parse_query(&format!("parent_id:{parent_id}"))?;

        // self.top_comments_with_count(
        //     limit,
        //     offset,
        //     query,
        //     Some(|top_docs: TopDocs| Box::new(top_docs.order_by_u64_field(ITEM_RANK, Order::Asc))),
        // )
        let searcher = self.searcher();

        let mut multi_collector = MultiCollector::new();

        let top_docs = TopDocs::with_limit(limit)
            .and_offset(offset)
            .order_by_u64_field(ITEM_RANK, Order::Asc);

        let docs_handle = multi_collector.add_collector(top_docs);
        let count_handle = multi_collector.add_collector(Count);

        let mut multi_fruit = searcher.search(&query, &multi_collector)?;
        let docs = docs_handle.extract(&mut multi_fruit);
        let count = count_handle.extract(&mut multi_fruit);

        let comments = docs
            .into_iter()
            .map(|(_, doc_address)| {
                let doc = searcher.doc::<TantivyDocument>(doc_address)?;
                self.to_comment(doc)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok((comments, count))
    }

    /// Search user comments with term, related story, limit and pagination offset.
    pub fn search_comments(
        &self,
        search: &str,
        story_id: u64,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Comment>, usize), SearchError> {
        let parent_query = Box::new(TermQuery::new(
            Term::from_field_u64(self.schema.get_field(ITEM_STORY_ID)?, story_id),
            IndexRecordOption::Basic,
        ));

        let fuzzy_search = Box::new(FuzzyTermQuery::new(
            Term::from_field_text(self.schema.get_field(ITEM_BODY)?, search),
            1,
            true,
        ));
        // let term_search = Box::new(TermQuery::new(
        //     Term::from_field_text(self.schema.get_field(ITEM_BODY)?, search),
        //     IndexRecordOption::Basic,
        // ));

        let combined_query = BooleanQuery::new(vec![
            (Occur::Must, parent_query),
            (Occur::Must, fuzzy_search),
            // (Occur::Must, term_search),
        ]);

        self.top_comments_with_count(limit, offset, combined_query)
    }

    /// Search all comments across all stories with limit and pagination offset.
    pub fn search_all_comments(
        &self,
        search: &str,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<Comment>, usize), SearchError> {
        let parsed_query = self.query_parser()?.parse_query(search)?;

        let type_query = TermQuery::new(
            Term::from_field_text(self.schema.get_field(ITEM_TYPE)?, "comment"),
            IndexRecordOption::Basic,
        );

        let query = BooleanQuery::new(vec![
            (Occur::Must, Box::new(type_query)),
            (Occur::Must, parsed_query),
        ]);

        self.top_comments_with_count(limit, offset, query)
    }

    /// Search query returning the total count and matching documents within
    /// offset and limit.
    fn top_comments_with_count(
        &self,
        limit: usize,
        offset: usize,
        query: impl Query,
    ) -> Result<(Vec<Comment>, usize), SearchError> {
        let searcher = self.searcher();

        let mut multi_collector = MultiCollector::new();

        // let top_docs = match order_fn {
        //     Some(f) => f(TopDocs::with_limit(limit).and_offset(offset)),
        //     None => Box::new(TopDocs::with_limit(limit).and_offset(offset)),
        // };
        let top_docs = TopDocs::with_limit(limit).and_offset(offset);

        let docs_handle = multi_collector.add_collector(top_docs);
        let count_handle = multi_collector.add_collector(Count);

        let mut multi_fruit = searcher.search(&query, &multi_collector)?;
        let docs = docs_handle.extract(&mut multi_fruit);
        let count = count_handle.extract(&mut multi_fruit);

        let comments = docs
            .into_iter()
            .map(|(_, doc_address)| {
                let doc = searcher.doc::<TantivyDocument>(doc_address)?;
                self.to_comment(doc)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((comments, count))
    }

    /// Build a comment stack by walking up the tree of nested comments.
    pub fn parents(&self, comment_id: u64) -> Result<CommentStack, SearchError> {
        let searcher = self.searcher();

        let comment = self.comment(&searcher, comment_id)?;
        let story_id = comment.story_id;

        let mut parent_id = (comment.parent_id != comment.story_id).then_some(comment.parent_id);
        let mut parents = Vec::from_iter([comment]);

        while let Some(id) = parent_id.take() {
            let next = self.comment(&searcher, id)?;
            parent_id = (next.parent_id != next.story_id).then_some(next.parent_id);
            parents.push(next);
        }

        Ok(CommentStack {
            story: self.story(story_id)?,
            comments: parents,
        })
    }

    /// Get a single comment.
    fn comment(&self, searcher: &Searcher, comment_id: u64) -> Result<Comment, SearchError> {
        let top_docs = TopDocs::with_limit(1);
        let parent_query: Box<dyn Query> = Box::new(TermQuery::new(
            Term::from_field_u64(self.schema.get_field(ITEM_ID)?, comment_id),
            IndexRecordOption::Basic,
        ));

        let (_score, doc_address) = searcher
            .search(&parent_query, &top_docs)?
            .into_iter()
            .next()
            .ok_or_else(|| SearchError::MissingDoc)?;

        searcher
            .doc::<TantivyDocument>(doc_address)
            .map(|doc| self.to_comment(doc))?
    }
}

#[derive(Debug)]
pub struct CommentStack {
    pub comments: Vec<Comment>,
    pub story: Story,
}
