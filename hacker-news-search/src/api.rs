use crate::{
    SearchContext, SearchError, SearchResult, ITEM_BODY, ITEM_BY, ITEM_DESCENDANT_COUNT, ITEM_ID,
    ITEM_KIDS, ITEM_PARENT_ID, ITEM_RANK, ITEM_SCORE, ITEM_STORY_ID, ITEM_TIME, ITEM_TITLE,
    ITEM_TYPE, ITEM_URL,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tantivy::{
    schema::{document::CompactDocValue, Value},
    Document, TantivyDocument,
};

mod comment;
mod story;

pub use comment::CommentStack;

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
    /// Rank
    pub rank: u64,
}

impl Story {
    pub fn age_label(&self) -> Option<String> {
        let duration = DateTime::<Utc>::from_timestamp(self.time.try_into().ok()?, 0)
            .map(|then| Utc::now() - then)?;

        let hours = duration.num_hours();
        let minutes = duration.num_minutes();
        let days = duration.num_days();

        match (days, hours, minutes) {
            (0, 0, 1) => "1 minute ago".to_string(),
            (0, 0, m) => format!("{m} minutes ago"),
            (0, 1, _) => "1 hour ago".to_string(),
            (0, h, _) => format!("{h} hours ago"),
            (1, _, _) => "1 day ago".to_string(),
            (d, _, _) => format!("{d} days ago"),
        }
        .into()
    }
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
    /// Parent story
    pub story_id: u64,
    /// Parent comment or story
    pub parent_id: u64,
    /// Rank
    pub rank: u64,
}

impl SearchContext {
    fn to_story(&self, doc: TantivyDocument) -> SearchResult<Story> {
        let mut fields = self.extract_fields(&doc);

        Ok(Story {
            id: fields
                .remove(ITEM_ID)
                .and_then(u64_value)
                .ok_or_else(|| missing_field(ITEM_ID))?,
            title: fields
                .remove(ITEM_TITLE)
                .and_then(str_value)
                .ok_or_else(|| missing_field(ITEM_TITLE))?,
            body: fields.remove(ITEM_BODY).and_then(str_value),
            url: fields.remove(ITEM_URL).and_then(str_value),
            by: fields
                .remove(ITEM_BY)
                .and_then(str_value)
                .ok_or_else(|| missing_field(ITEM_BY))?,
            ty: fields
                .remove(ITEM_TYPE)
                .and_then(str_value)
                .ok_or_else(|| missing_field(ITEM_TYPE))?,
            descendants: fields
                .remove(ITEM_DESCENDANT_COUNT)
                .and_then(u64_value)
                .unwrap_or_default(),
            time: fields
                .remove(ITEM_TIME)
                .and_then(u64_value)
                .ok_or_else(|| missing_field(ITEM_TIME))?,
            score: fields.remove(ITEM_SCORE).and_then(u64_value).unwrap_or(1),
            rank: fields
                .remove(ITEM_RANK)
                .and_then(u64_value)
                .ok_or_else(|| missing_field(ITEM_RANK))?,
        })
    }

    fn to_comment(&self, doc: TantivyDocument) -> SearchResult<Comment> {
        let mut fields = self.extract_fields(&doc);

        Ok(Comment {
            id: fields
                .remove(ITEM_ID)
                .and_then(u64_value)
                .ok_or_else(|| missing_field(ITEM_ID))?,
            body: fields
                .remove(ITEM_BODY)
                .and_then(str_value)
                .ok_or_else(|| missing_field(ITEM_BODY))?,
            by: fields
                .remove(ITEM_BY)
                .and_then(str_value)
                .ok_or_else(|| missing_field(ITEM_BY))?,
            time: fields
                .remove(ITEM_TIME)
                .and_then(u64_value)
                .ok_or_else(|| missing_field(ITEM_TIME))?,
            kids: fields.remove(ITEM_KIDS).map(u64_values).unwrap_or_default(),
            story_id: fields
                .remove(ITEM_STORY_ID)
                .and_then(u64_value)
                .ok_or_else(|| missing_field(ITEM_STORY_ID))?,
            parent_id: fields
                .remove(ITEM_PARENT_ID)
                .and_then(u64_value)
                .ok_or_else(|| missing_field(ITEM_PARENT_ID))?,
            rank: fields
                .remove(ITEM_RANK)
                .and_then(u64_value)
                .ok_or_else(|| missing_field(ITEM_RANK))?,
        })
    }

    fn extract_fields<'a>(
        &'a self,
        doc: &'a TantivyDocument,
    ) -> HashMap<&'a str, Vec<CompactDocValue<'a>>> {
        doc.get_sorted_field_values()
            .into_iter()
            .flat_map(|(field, field_values)| {
                let field_name = self.schema.get_field_name(field);
                Some((field_name, field_values))
            })
            .collect()
    }
}

fn str_value(mut owned_value: Vec<CompactDocValue<'_>>) -> Option<String> {
    owned_value.pop()?.as_str().map(ToOwned::to_owned)
}

fn u64_value(mut owned_value: Vec<CompactDocValue<'_>>) -> Option<u64> {
    owned_value.pop()?.as_u64()
}

fn u64_values(owned_value: Vec<CompactDocValue<'_>>) -> Vec<u64> {
    owned_value
        .into_iter()
        .filter_map(|value| value.as_u64())
        .collect()
}

fn missing_field(field: &str) -> SearchError {
    SearchError::MissingField(field.into())
}
