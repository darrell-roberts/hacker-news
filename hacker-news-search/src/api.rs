use crate::{
    SearchContext, SearchError, ITEM_BODY, ITEM_BY, ITEM_DESCENDANT_COUNT, ITEM_ID, ITEM_KIDS,
    ITEM_PARENT_ID, ITEM_SCORE, ITEM_STORY_ID, ITEM_TIME, ITEM_TITLE, ITEM_TYPE, ITEM_URL,
};
use std::collections::HashMap;
use tantivy::{schema::OwnedValue, Document, TantivyDocument};

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
}

impl SearchContext {
    fn to_story(&self, doc: TantivyDocument) -> Result<Story, SearchError> {
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
        })
    }

    fn to_comment(&self, doc: TantivyDocument) -> Result<Comment, SearchError> {
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

fn missing_field(field: &str) -> SearchError {
    SearchError::MissingField(field.into())
}
