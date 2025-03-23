//! content that is used on the history stack.
use crate::{
    comments::{CommentState, NavStack},
    full_search::{FullSearchState, SearchCriteria},
};
use hacker_news_search::{api::CommentStack, SearchContext};
use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

/// Type of content to render in the content pane.
pub enum Content {
    /// User comments
    Comment(CommentState),
    /// Comment search
    Search(FullSearchState),
    /// Empty
    Empty,
}

impl Content {
    /// Convert this content into a history element.
    pub fn into_history_element(self) -> HistoryElement {
        match self {
            Content::Comment(comment_state) => comment_state.to_history().into(),
            Content::Search(full_search_state) => full_search_state.to_history().into(),
            Content::Empty => HistoryElement::Empty,
        }
    }

    /// Get the active story to show for this content.
    pub fn active_story(&self) -> Option<u64> {
        match self {
            Content::Comment(comment_state) => Some(comment_state.article.id),
            Content::Search(full_search_state) => match &full_search_state.search {
                SearchCriteria::Query(_) => None,
                SearchCriteria::StoryId { story_id, .. } => Some(*story_id),
            },
            Content::Empty => None,
        }
    }
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

/// Convert state into a history item and from a history item.
pub trait History
where
    Self: Sized,
{
    type HistoryItem;

    /// Restore state from a history item.
    fn from_history(
        search_context: Arc<RwLock<SearchContext>>,
        item: Self::HistoryItem,
    ) -> anyhow::Result<Self>;

    /// Save state into a a history item.
    fn to_history(self) -> Self::HistoryItem;
}

/// A history element
pub enum HistoryElement {
    /// History for the comments state
    Comment(CommentHistory),
    /// History for the search state
    Search(SearchHistory),
    /// History for no state
    Empty,
}

impl HistoryElement {
    pub fn into_content(
        self,
        search_context: Arc<RwLock<SearchContext>>,
    ) -> anyhow::Result<Content> {
        Ok(match self {
            HistoryElement::Comment(comment_history) => {
                Content::Comment(CommentState::from_history(search_context, comment_history)?)
            }
            HistoryElement::Search(search_history) => Content::Search(
                FullSearchState::from_history(search_context, search_history)?,
            ),
            HistoryElement::Empty => Content::Empty,
        })
    }
}

impl From<CommentHistory> for HistoryElement {
    fn from(history: CommentHistory) -> Self {
        Self::Comment(history)
    }
}

impl From<SearchHistory> for HistoryElement {
    fn from(history: SearchHistory) -> Self {
        Self::Search(history)
    }
}

pub struct CommentHistory {
    story_id: u64,
    search: Option<String>,
    oneline: bool,
    offset: usize,
    page: usize,
    parent_id: u64,
    active_comment_id: Option<u64>,
}

impl History for CommentState {
    type HistoryItem = CommentHistory;

    fn from_history(
        search_context: Arc<RwLock<SearchContext>>,
        item: Self::HistoryItem,
    ) -> anyhow::Result<Self> {
        let ctx = search_context.clone();
        let sc = ctx.read().unwrap();
        let (mut comments, total_comments) = sc.comments(item.parent_id, 10, item.offset)?;

        let nav_stack = if let Some(viewing_id) = item.active_comment_id {
            let CommentStack {
                comments: mut comment_stack,
                ..
            } = sc.parents(viewing_id)?;
            // Take the last comment which is the first comment.
            comments = comment_stack.pop().map(|c| vec![c]).unwrap_or_default();

            comment_stack.reverse();
            let mut nav_stack = vec![NavStack::root()];
            nav_stack.extend(comment_stack.into_iter().map(|comment| NavStack {
                comment: Some(comment),
                offset: 0,
                page: 1,
                scroll_offset: None,
            }));
            nav_stack
        } else {
            Vec::new()
        };

        let article = sc.story(item.story_id)?;

        Ok(Self {
            search_context,
            article,
            nav_stack,
            comments,
            search: item.search,
            oneline: item.oneline,
            offset: item.offset,
            page: item.page,
            full_count: total_comments,
            parent_id: item.parent_id,
            active_comment_id: item.active_comment_id,
        })
    }

    fn to_history(self) -> Self::HistoryItem {
        Self::HistoryItem {
            story_id: self.article.id,
            search: self.search,
            oneline: self.oneline,
            offset: self.offset,
            page: self.page,
            parent_id: self.parent_id,
            active_comment_id: self.active_comment_id,
        }
    }
}

pub struct SearchHistory {
    search: SearchCriteria,
    offset: usize,
    page: usize,
}

impl History for FullSearchState {
    type HistoryItem = SearchHistory;

    fn from_history(
        search_context: Arc<RwLock<SearchContext>>,
        item: Self::HistoryItem,
    ) -> anyhow::Result<Self> {
        let ctx = search_context.clone();

        let (search_results, full_count) = match &item.search {
            SearchCriteria::Query(s) => {
                let g = ctx.read().unwrap();
                g.search_all_comments(s, 10, item.offset)?
            }
            SearchCriteria::StoryId { story_id, beyond } => {
                let g = ctx.read().unwrap();
                g.story_comments_by_date(*story_id, *beyond, 10, item.offset)?
            }
        };

        let state = FullSearchState {
            search: item.search,
            search_results,
            search_context,
            offset: item.offset,
            page: item.page,
            full_count,
        };

        Ok(state)
    }

    fn to_history(self) -> Self::HistoryItem {
        Self::HistoryItem {
            search: self.search,
            offset: self.offset,
            page: self.page,
        }
    }
}
