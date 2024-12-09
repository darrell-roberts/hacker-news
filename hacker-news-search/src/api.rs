use crate::{
    SearchContext, SearchError, ITEM_BODY, ITEM_BY, ITEM_CATEGORY, ITEM_DESCENDANT_COUNT, ITEM_ID,
    ITEM_KIDS, ITEM_RANK, ITEM_SCORE, ITEM_STORY_ID, ITEM_TIME, ITEM_TITLE, ITEM_TYPE, ITEM_URL,
};
use std::collections::HashMap;
use tantivy::{
    collector::TopDocs,
    query::{BooleanQuery, FuzzyTermQuery, Occur, TermQuery},
    schema::{IndexRecordOption, OwnedValue},
    Document, Order, TantivyDocument, Term,
};

impl SearchContext {
    pub fn top_stories(&self, offset: usize) -> Result<Vec<Story>, SearchError> {
        let query = self
            .query_parser()?
            .parse_query("category:top AND type:story")?;
        let searcher = self.searcher();

        let top_docs = TopDocs::with_limit(50)
            // Pagination
            .and_offset(offset)
            // Ordering
            .order_by_u64_field(ITEM_RANK, Order::Asc);

        let docs = searcher
            .search(&query, &top_docs)?
            .into_iter()
            .map(|(_, doc)| searcher.doc::<TantivyDocument>(doc))
            .collect::<Result<Vec<_>, _>>()?;

        let stories = docs
            .into_iter()
            .flat_map(|doc| self.to_story(doc))
            .collect::<Vec<_>>();

        Ok(stories)
    }

    pub fn comments(&self, parent_id: u64, offset: usize) -> Result<Vec<Comment>, SearchError> {
        let query = self
            .query_parser()?
            .parse_query(&format!("parent_id:{parent_id}"))?;
        let searcher = self.searcher();

        let top_docs = TopDocs::with_limit(50)
            // Pagination
            .and_offset(offset)
            // Ordering
            .order_by_u64_field(ITEM_RANK, Order::Asc);

        let docs = searcher
            .search(&query, &top_docs)?
            .into_iter()
            .map(|(_, doc)| searcher.doc::<TantivyDocument>(doc))
            .collect::<Result<Vec<_>, _>>()?;

        let comments = docs
            .into_iter()
            .flat_map(|doc| self.to_comment(doc))
            .collect::<Vec<_>>();

        Ok(comments)
    }

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
        let top_docs = TopDocs::with_limit(50)
            // Pagination
            .and_offset(offset)
            // Ordering
            .order_by_u64_field(ITEM_RANK, Order::Asc);

        let docs = searcher
            .search(&query, &top_docs)?
            .into_iter()
            .map(|(_, doc)| searcher.doc::<TantivyDocument>(doc))
            .collect::<Result<Vec<_>, _>>()?;

        let stories = docs
            .into_iter()
            .flat_map(|doc| self.to_story(doc))
            .collect::<Vec<_>>();

        Ok(stories)
    }

    pub fn search_comments(
        &self,
        search: &str,
        story_id: u64,
        offset: usize,
    ) -> Result<Vec<Comment>, SearchError> {
        let searcher = self.searcher();

        let parent_query = Box::new(TermQuery::new(
            Term::from_field_u64(self.schema.get_field(ITEM_STORY_ID)?, story_id),
            IndexRecordOption::Basic,
        ));

        let fuzzy_search = Box::new(FuzzyTermQuery::new(
            Term::from_field_text(self.schema.get_field(ITEM_BODY)?, search),
            1,
            true,
        ));

        let combined_query = BooleanQuery::new(vec![
            (Occur::Must, parent_query),
            (Occur::Must, fuzzy_search),
        ]);

        let top_docs = TopDocs::with_limit(50)
            // Pagination
            .and_offset(offset)
            // Ordering
            .order_by_u64_field(ITEM_TIME, Order::Asc);

        let docs = searcher
            .search(&combined_query, &top_docs)?
            .into_iter()
            .map(|(_, doc)| searcher.doc::<TantivyDocument>(doc))
            .collect::<Result<Vec<_>, _>>()?;

        let comments = docs
            .into_iter()
            .flat_map(|doc| self.to_comment(doc))
            .collect::<Vec<_>>();

        Ok(comments)
    }

    pub fn search_all_comments(
        &self,
        search: &str,
        offset: usize,
    ) -> Result<Vec<Comment>, SearchError> {
        let searcher = self.searcher();

        // let query = TermQuery::new(
        //     Term::from_field_text(self.schema.get_field(ITEM_BODY)?, search),
        //     IndexRecordOption::WithFreqsAndPositions, // 1,
        //                                               // true,
        // );
        let query = self.query_parser()?.parse_query(search)?;

        let top_docs = TopDocs::with_limit(50)
            // Pagination
            .and_offset(offset)
            // Ordering
            .order_by_u64_field(ITEM_TIME, Order::Asc);

        let docs = searcher
            .search(&query, &top_docs)?
            .into_iter()
            .map(|(_, doc)| searcher.doc::<TantivyDocument>(doc))
            .collect::<Result<Vec<_>, _>>()?;

        let comments = docs
            .into_iter()
            .flat_map(|doc| self.to_comment(doc))
            .collect::<Vec<_>>();

        Ok(comments)
    }
}

#[derive(Debug, Clone)]
/// Hacker news story
pub struct Story {
    /// Id
    pub id: u64,
    /// Title
    pub title: String,
    /// Body
    pub body: Option<String>,
    /// Url
    pub url: Option<String>,
    /// By
    pub by: String,
    /// Type
    pub ty: String,
    /// Descendant count
    pub descendants: u64,
    /// Time posted
    pub time: u64,
    /// Score
    pub score: u64,
}

#[derive(Debug, Clone)]
/// Hacker news comment
pub struct Comment {
    /// Id
    pub id: u64,
    /// Body
    pub body: String,
    /// By
    pub by: String,
    /// Time posted
    pub time: u64,
    /// Kids
    pub kids: Vec<u64>,
}

impl SearchContext {
    fn to_story(&self, doc: TantivyDocument) -> Option<Story> {
        let mut fields = self.extract_fields(&doc);

        Some(Story {
            id: fields.remove(ITEM_ID).and_then(u64_value)?,
            title: fields.remove(ITEM_TITLE).and_then(str_value)?,
            body: fields.remove(ITEM_BODY).and_then(str_value),
            url: fields.remove(ITEM_URL).and_then(str_value),
            by: fields.remove(ITEM_BY).and_then(str_value)?,
            ty: fields.remove(ITEM_TYPE).and_then(str_value)?,
            descendants: fields.remove(ITEM_DESCENDANT_COUNT).and_then(u64_value)?,
            time: fields.remove(ITEM_TIME).and_then(u64_value)?,
            score: fields.remove(ITEM_SCORE).and_then(u64_value)?,
        })
    }

    fn to_comment(&self, doc: TantivyDocument) -> Option<Comment> {
        let mut fields = self.extract_fields(&doc);

        Some(Comment {
            id: fields.remove(ITEM_ID).and_then(u64_value)?,
            body: fields.remove(ITEM_BODY).and_then(str_value)?,
            by: fields.remove(ITEM_BY).and_then(str_value)?,
            time: fields.remove(ITEM_TIME).and_then(u64_value)?,
            kids: fields.remove(ITEM_KIDS).map(u64_values)?,
        })
    }

    fn extract_fields<'a>(
        &'a self,
        doc: &'a TantivyDocument,
    ) -> HashMap<&'a str, Vec<&'a OwnedValue>> {
        doc.get_sorted_field_values()
            .into_iter()
            .flat_map(|(field, field_values)| {
                let field_name = self.schema.get_field_name(field);
                Some((field_name, field_values))
            })
            .collect()
    }
}

fn str_value(mut owned_value: Vec<&OwnedValue>) -> Option<String> {
    let single_value = owned_value.pop()?;
    match single_value {
        OwnedValue::Str(s) => Some(s.to_owned()),
        _ => None,
    }
}

fn u64_value(mut owned_value: Vec<&OwnedValue>) -> Option<u64> {
    let single_value = owned_value.pop()?;
    match single_value {
        OwnedValue::U64(n) => Some(*n),
        _ => None,
    }
}

fn u64_values(owned_value: Vec<&OwnedValue>) -> Vec<u64> {
    owned_value
        .into_iter()
        .filter_map(|value| match value {
            OwnedValue::U64(n) => Some(*n),
            _ => None,
        })
        .collect()
}
