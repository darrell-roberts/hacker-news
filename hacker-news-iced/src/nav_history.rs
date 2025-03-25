//! content that is used on the history stack.
use crate::{
    comments::{CommentState, NavStack},
    full_search::{FullSearchState, SearchCriteria},
};
use anyhow::Context;
use hacker_news_api::ArticleType;
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
    Empty(ArticleType),
}

impl Content {
    /// Convert this content into a history element.
    pub fn into_history_element(self) -> HistoryElement {
        match self {
            Content::Comment(comment_state) => comment_state.to_history().into(),
            Content::Search(full_search_state) => full_search_state.to_history().into(),
            Content::Empty(index) => HistoryElement::Empty(index),
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
            Content::Empty(_) => None,
        }
    }

    /// Get the search text.
    pub fn search_text(&self) -> Option<String> {
        match self {
            Content::Search(full_search_state) => match &full_search_state.search {
                SearchCriteria::Query(search) => Some(search.to_owned()),
                SearchCriteria::StoryId { .. } => None,
            },
            _ => None,
        }
    }
}

impl Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Comment(_) => f.write_str("Comments"),
            Content::Search(_) => f.write_str("Search"),
            Content::Empty(index) => write!(f, "Empty for {index}"),
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
    ) -> anyhow::Result<(ArticleType, Self)>;

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
    Empty(ArticleType),
}

impl HistoryElement {
    /// Convert self into [`Content`].
    pub fn into_content(
        self,
        search_context: Arc<RwLock<SearchContext>>,
    ) -> anyhow::Result<(ArticleType, Content)> {
        Ok(match self {
            HistoryElement::Comment(comment_history) => {
                let (index, comment_state) =
                    CommentState::from_history(search_context, comment_history)?;
                (index, Content::Comment(comment_state))
            }
            HistoryElement::Search(search_history) => {
                let (index, search_state) =
                    FullSearchState::from_history(search_context, search_history)?;
                (index, Content::Search(search_state))
            }
            HistoryElement::Empty(index) => {
                search_context.write().unwrap().activate_index(index)?;
                (index, Content::Empty(index))
            }
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

/// History for the comment state.
pub struct CommentHistory {
    story_id: u64,
    search: Option<String>,
    oneline: bool,
    offset: usize,
    page: usize,
    parent_id: u64,
    active_comment_id: Option<u64>,
    category: ArticleType,
}

impl History for CommentState {
    type HistoryItem = CommentHistory;

    fn from_history(
        search_context: Arc<RwLock<SearchContext>>,
        item: Self::HistoryItem,
    ) -> anyhow::Result<(ArticleType, Self)> {
        let ctx = search_context.clone();
        let mut sc = ctx.write().unwrap();
        if sc.active_category() != item.category {
            log::debug!("Switching active index to {}", item.category);
            sc.activate_index(item.category)?;
        }
        let (mut comments, total_comments) = sc
            .comments(item.parent_id, 10, item.offset)
            .with_context(|| {
                format!(
                    "Could not lookup {} in index {}",
                    item.parent_id, item.category
                )
            })?;

        let nav_stack = match item.active_comment_id {
            Some(viewing_id) => {
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
            }
            None => Vec::new(),
        };

        let article = sc.story(item.story_id)?;

        Ok((
            item.category,
            Self {
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
            },
        ))
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
            category: self.search_context.read().unwrap().active_category(),
        }
    }
}

/// History for the search state.
pub struct SearchHistory {
    search: SearchCriteria,
    offset: usize,
    page: usize,
    category: ArticleType,
}

impl History for FullSearchState {
    type HistoryItem = SearchHistory;

    fn from_history(
        search_context: Arc<RwLock<SearchContext>>,
        item: Self::HistoryItem,
    ) -> anyhow::Result<(ArticleType, Self)> {
        let ctx = search_context.clone();
        let mut sc = ctx.write().unwrap();
        if sc.active_category() != item.category {
            log::debug!("Switching active index to {}", item.category);
            sc.activate_index(item.category)?;
        }
        let (search_results, full_count) = match &item.search {
            SearchCriteria::Query(s) => sc.search_all_comments(s, 10, item.offset)?,
            SearchCriteria::StoryId { story_id, beyond } => {
                sc.story_comments_by_date(*story_id, *beyond, 10, item.offset)?
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

        Ok((item.category, state))
    }

    fn to_history(self) -> Self::HistoryItem {
        Self::HistoryItem {
            search: self.search,
            offset: self.offset,
            page: self.page,
            category: self.search_context.read().unwrap().active_category(),
        }
    }
}
