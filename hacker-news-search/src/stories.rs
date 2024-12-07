use crate::{
    SearchContext, SearchError, ITEM_BODY, ITEM_BY, ITEM_DESCENDANT_COUNT, ITEM_ID, ITEM_RANK,
    ITEM_TIME, ITEM_TITLE, ITEM_TYPE, ITEM_URL,
};
use std::collections::HashMap;
use tantivy::{collector::TopDocs, schema::OwnedValue, Document, Order, TantivyDocument};

impl SearchContext {
    pub fn top_stories(&self, offset: usize) -> Result<Vec<Story>, SearchError> {
        let query = self
            .query_parser()?
            .parse_query("category:top AND type:story")?;
        let searcher = self.searcher();

        let top_docs = TopDocs::with_limit(10)
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

        let top_docs = TopDocs::with_limit(10)
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

#[derive(Debug)]
pub struct Story {
    id: u64,
    title: String,
    body: Option<String>,
    url: Option<String>,
    by: String,
    ty: String,
    descendants: u64,
    time: u64,
}

#[derive(Debug)]
pub struct Comment {
    id: u64,
    body: String,
    by: String,
    time: u64,
}

impl SearchContext {
    fn to_story(&self, doc: TantivyDocument) -> Option<Story> {
        let fields = self.extract_fields(&doc);

        Some(Story {
            id: fields.get(ITEM_ID).and_then(u64_value)?,
            title: fields.get(ITEM_TITLE).and_then(str_value)?,
            body: fields.get(ITEM_BODY).and_then(str_value),
            url: fields.get(ITEM_URL).and_then(str_value),
            by: fields.get(ITEM_BY).and_then(str_value)?,
            ty: fields.get(ITEM_TYPE).and_then(str_value)?,
            descendants: fields.get(ITEM_DESCENDANT_COUNT).and_then(u64_value)?,
            time: fields.get(ITEM_TIME).and_then(u64_value)?,
        })
    }

    fn to_comment(&self, doc: TantivyDocument) -> Option<Comment> {
        let fields = self.extract_fields(&doc);

        Some(Comment {
            id: fields.get(ITEM_ID).and_then(u64_value)?,
            body: fields.get(ITEM_BODY).and_then(str_value)?,
            by: fields.get(ITEM_BY).and_then(str_value)?,
            time: fields.get(ITEM_TIME).and_then(u64_value)?,
        })
    }

    fn extract_fields<'a>(&'a self, doc: &'a TantivyDocument) -> HashMap<&'a str, &'a OwnedValue> {
        doc.get_sorted_field_values()
            .into_iter()
            .flat_map(|(field, mut field_values)| {
                let field_name = self.schema.get_field_name(field);
                let value = field_values.pop()?;
                Some((field_name, value))
            })
            .collect()
    }
}

fn str_value(owned_value: &&OwnedValue) -> Option<String> {
    match owned_value {
        OwnedValue::Str(s) => Some(s.to_owned()),
        _ => None,
    }
}

fn u64_value(owned_value: &&OwnedValue) -> Option<u64> {
    match owned_value {
        OwnedValue::U64(n) => Some(*n),
        _ => None,
    }
}
