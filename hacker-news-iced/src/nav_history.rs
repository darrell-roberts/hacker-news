//! content that is used on the history stack.
use crate::{comments::CommentState, full_search::FullSearchState};
use std::fmt::Display;

pub enum Content {
    Comment(CommentState),
    Search(FullSearchState),
    Empty,
}

impl Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Comment(_) => f.write_str("Comments"),
            Content::Search(_) => f.write_str("Search"),
            Content::Empty => f.write_str("Empty"),
        }
    }
}
