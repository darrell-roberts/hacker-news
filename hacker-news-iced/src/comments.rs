//! State and view for viewing comments.
use crate::{
    app::AppMsg,
    common::{error_task, PaginatingView},
    full_search::FullSearchMsg,
    header::HeaderMsg,
    parse_date,
    richtext::render_rich_text,
};
use hacker_news_search::{
    api::{Comment, Story},
    SearchContext,
};
use iced::{
    border,
    font::{Style, Weight},
    padding,
    widget::{
        self, button, container, scrollable::AbsoluteOffset, text::Shaping, Column, Container,
    },
    Border, Element, Font, Length, Task,
};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
/// A navigation stack element.
pub struct NavStack {
    /// Parent comment
    pub comment: Option<Comment>,
    /// Pagination offset
    pub offset: usize,
    /// Viewing page
    pub page: usize,
    /// Scroll offset
    pub scroll_offset: Option<AbsoluteOffset>,
}

/// Comment state
pub struct CommentState {
    pub search_context: Arc<RwLock<SearchContext>>,
    /// Article this comment belongs to
    pub article: Story,
    /// parent comments.
    pub nav_stack: Vec<NavStack>,
    /// Children
    pub comments: Vec<Comment>,
    /// Search
    pub search: Option<String>,
    /// Show one line only.
    pub oneline: bool,
    /// Search offset.
    pub offset: usize,
    /// Search page number.
    pub page: usize,
    /// Total number of documents.
    pub full_count: usize,
    /// Search results
    pub search_results: Vec<Comment>,
    /// Parent id being viewed.
    pub parent_id: u64,
}

#[derive(Debug, Clone)]
pub enum CommentMsg {
    FetchComments {
        parent_id: u64,
        parent_comment: Option<Comment>,
        scroll_to: Option<AbsoluteOffset>,
    },
    PopNavStack,
    Search(String),
    OpenSearch,
    CloseSearch,
    Oneline,
    Forward,
    Back,
    JumpPage(usize),
    Close(u64),
    ScrollOffset(AbsoluteOffset),
}

impl CommentState {
    /// Render the comments
    pub fn view(&self) -> Element<'_, AppMsg> {
        let article_text = self
            .article
            .body
            .as_deref()
            .map(|text| widget::rich_text(render_rich_text(text, self.search.as_deref(), false)))
            .map(|rt| container(rt).padding([10, 10]).into());

        let comment_rows = self
            .comments
            .iter()
            .map(|item| {
                self.render_comment(item, false).style(|theme| {
                    let palette = theme.extended_palette();

                    container::Style {
                        background: Some(palette.background.weak.color.into()),
                        border: border::rounded(8),
                        ..Default::default()
                    }
                })
            })
            .map(Element::from)
            .collect::<Vec<_>>();

        let parent_comments = self
            .nav_stack
            .iter()
            .filter_map(|stack| stack.comment.as_ref())
            .map(|parent| {
                self.render_comment(parent, true).style(|theme| {
                    let palette = theme.extended_palette();

                    container::Style {
                        border: Border {
                            color: palette.secondary.weak.color,
                            width: 1.,
                            radius: 8.into(),
                        },
                        ..Default::default()
                    }
                })
            })
            .map(Element::from);

        let content = Column::new()
            .push_maybe(self.search.as_ref().map(|search| {
                widget::Row::new()
                    .push(
                        widget::text_input("Search...", search)
                            .id(widget::text_input::Id::new("comment_search"))
                            .on_input(|input| AppMsg::Comments(CommentMsg::Search(input))),
                    )
                    .push(widget::tooltip(
                        widget::button(widget::text("⟲").shaping(Shaping::Advanced))
                            .on_press(AppMsg::Comments(CommentMsg::CloseSearch)),
                        widget::container(widget::text("Clear search")).padding(5),
                        widget::tooltip::Position::Left,
                    ))
            }))
            .push_maybe((self.full_count > 10).then(|| self.pagination_element()))
            .push(
                widget::scrollable(
                    widget::Column::new()
                        .push_maybe(
                            self.search
                                .is_none()
                                .then(|| Column::with_children(article_text).spacing(15)),
                        )
                        .push_maybe(
                            self.search
                                .is_none()
                                .then(|| Column::with_children(parent_comments).spacing(15)),
                        )
                        .push(Column::with_children(comment_rows).spacing(15))
                        .spacing(15)
                        .padding(padding::top(0).bottom(10).left(10).right(25)),
                )
                .on_scroll(|viewport| {
                    AppMsg::Comments(CommentMsg::ScrollOffset(viewport.absolute_offset()))
                })
                .id(comment_scroll_id())
                .height(Length::Fill),
            );

        container(content.width(Length::Fill)).into()
    }

    /// Render a single comment
    fn render_comment<'a>(
        &'a self,
        comment: &'a Comment,
        is_parent: bool,
    ) -> Container<'a, AppMsg> {
        let by_button: Element<'_, AppMsg> = if comment.kids.is_empty() {
            widget::text("").into()
        } else {
            button(widget::text(format!("💬{}", comment.kids.len())).shaping(Shaping::Advanced))
                .padding(0)
                .on_press(AppMsg::Comments(CommentMsg::FetchComments {
                    parent_id: comment.id,
                    parent_comment: Some(comment.clone()),
                    scroll_to: None,
                }))
                .style(button::text)
                .into()
        };

        container(
            widget::Column::new()
                .push_maybe(is_parent.then(|| {
                    widget::container(
                        widget::button("X")
                            .on_press(AppMsg::Comments(CommentMsg::Close(comment.id))),
                    )
                    .align_right(Length::Fill)
                }))
                .push(widget::rich_text(render_rich_text(
                    &comment.body,
                    self.search.as_deref(),
                    self.oneline,
                )))
                .push(
                    widget::row![
                        widget::rich_text([
                            widget::span(format!(" by {}", comment.by))
                                .link(AppMsg::Header(HeaderMsg::Search(format!(
                                    "by:{}",
                                    comment.by
                                ))))
                                .font(Font {
                                    style: Style::Italic,
                                    ..Default::default()
                                })
                                .size(14),
                            widget::span(" "),
                            widget::span(parse_date(comment.time).unwrap_or_default())
                                .font(Font {
                                    weight: Weight::Light,
                                    style: Style::Italic,
                                    ..Default::default()
                                })
                                .size(10),
                        ]),
                        by_button,
                        widget::container(
                            widget::button(widget::text(format!("{}", comment.id)))
                                .on_press(AppMsg::OpenLink {
                                    url: format!(
                                        "https://news.ycombinator.com/item?id={}",
                                        comment.id
                                    ),
                                    item_id: comment.story_id
                                })
                                .style(widget::button::text)
                                .padding(0)
                        )
                        .align_right(Length::Fill)
                    ]
                    .spacing(5),
                )
                .padding([10, 10])
                .spacing(15)
                .width(Length::Fill),
        )
        .clip(false)
    }

    /// Update comment viewing state.
    pub fn update(&mut self, message: CommentMsg) -> Task<AppMsg> {
        match message {
            CommentMsg::FetchComments {
                parent_id,
                parent_comment,
                scroll_to,
            } => {
                if let Some(parent) = parent_comment {
                    // We are viewing a nested comment
                    if self.parent_id != parent.id {
                        if let Some(index) =
                            self.nav_stack
                                .iter()
                                .enumerate()
                                .find_map(|(index, stack_item)| match stack_item.comment.as_ref() {
                                    Some(c) => (c.id == parent_id).then_some(index),
                                    None => None,
                                })
                        {
                            self.nav_stack.drain(index..);
                        }

                        self.nav_stack.push(NavStack {
                            comment: Some(parent),
                            offset: 0,
                            page: 1,
                            scroll_offset: None,
                        });
                        self.offset = 0;
                        self.page = 1;
                    }
                }
                let g = self.search_context.read().unwrap();
                let fetch_task = match g.comments(parent_id, 10, self.offset) {
                    Ok((comments, full_count)) => {
                        self.full_count = full_count;
                        self.comments = comments;

                        widget::scrollable::scroll_to(
                            comment_scroll_id(),
                            scroll_to.unwrap_or_default(),
                        )
                    }
                    Err(err) => error_task(err),
                };
                self.parent_id = parent_id;

                Task::batch([
                    fetch_task,
                    Task::done(FullSearchMsg::CloseSearch).map(AppMsg::FullSearch),
                ])
            }
            CommentMsg::PopNavStack => {
                self.nav_stack.pop();

                match self.nav_stack.last() {
                    Some(current) => {
                        self.offset = current.offset;
                        self.page = current.page;

                        Task::done(CommentMsg::FetchComments {
                            parent_id: current
                                .comment
                                .as_ref()
                                .map(|c| c.id)
                                .unwrap_or_else(|| self.article.id),
                            parent_comment: None,
                            scroll_to: current.scroll_offset,
                        })
                        .map(AppMsg::Comments)
                    }
                    None => Task::done(AppMsg::CommentsClosed),
                }
            }
            CommentMsg::Search(search) => {
                if search.is_empty() {
                    Task::done(AppMsg::Comments(CommentMsg::CloseSearch))
                } else {
                    match self.search.as_deref() {
                        // New search term.
                        Some(s) if s != search => {
                            if let Some(current) = self.nav_stack.last_mut() {
                                current.offset = 0;
                                current.page = 0;
                            }

                            self.offset = 0;
                            self.page = 1;
                        }
                        _ => (),
                    }

                    self.search = Some(search.clone());
                    let g = self.search_context.read().unwrap();
                    match g.search_comments(&search, self.article.id, 10, self.offset) {
                        Ok((comments, count)) => {
                            self.comments = comments;
                            self.full_count = count;
                            Task::none()
                        }
                        Err(err) => error_task(err),
                    }
                }
            }
            CommentMsg::OpenSearch => {
                self.search = Some(String::new());
                self.nav_stack.push(NavStack {
                    comment: None,
                    offset: 0,
                    page: 1,
                    scroll_offset: None,
                });
                self.offset = 0;
                self.page = 1;
                self.comments = Vec::new();
                widget::text_input::focus(widget::text_input::Id::new("comment_search"))
            }
            CommentMsg::CloseSearch => {
                self.search = None;
                self.search_results = Vec::new();

                Task::done(CommentMsg::PopNavStack).map(AppMsg::Comments)
            }
            CommentMsg::Oneline => {
                self.oneline = !self.oneline;
                Task::none()
            }
            CommentMsg::Forward => {
                self.offset += 10;
                self.page += 1;
                self.update_nav_stack();
                self.paginate_task()
            }
            CommentMsg::Back => {
                self.offset -= 10;
                self.page -= 1;
                self.update_nav_stack();
                self.paginate_task()
            }
            CommentMsg::JumpPage(page) => {
                self.page = page;
                if page > 1 {
                    self.offset = 10 * (page - 1);
                } else {
                    self.offset = 0;
                }
                self.update_nav_stack();
                self.paginate_task()
            }
            CommentMsg::Close(comment_id) => {
                while let Some(stack_item) = self.nav_stack.pop() {
                    if let Some(c) = stack_item.comment {
                        if c.id == comment_id {
                            break;
                        }
                    }
                }

                self.parent_id = self
                    .nav_stack
                    .last()
                    .map(|c| match c.comment.as_ref() {
                        Some(c) => c.parent_id,
                        None => 0,
                    })
                    .unwrap_or_default();

                match self.nav_stack.last() {
                    Some(current) => {
                        self.offset = current.offset;
                        self.page = current.page;

                        Task::done(CommentMsg::FetchComments {
                            parent_id: current
                                .comment
                                .as_ref()
                                .map(|c| c.id)
                                .unwrap_or_else(|| self.article.id),
                            parent_comment: None,
                            scroll_to: current.scroll_offset,
                        })
                        .map(AppMsg::Comments)
                    }
                    None => Task::none(),
                }
            }
            CommentMsg::ScrollOffset(offset) => {
                if let Some(current_nav) = self.nav_stack.last_mut() {
                    current_nav.scroll_offset = Some(offset);
                }
                Task::none()
            }
        }
    }

    fn paginate_task(&self) -> Task<AppMsg> {
        match self.search.as_ref() {
            Some(s) => Task::done(CommentMsg::Search(s.to_owned())).map(AppMsg::Comments),
            None => Task::done(CommentMsg::FetchComments {
                parent_id: self.parent_id,
                parent_comment: None,
                scroll_to: None,
            })
            .map(AppMsg::Comments),
        }
    }

    fn update_nav_stack(&mut self) {
        if let Some(current) = self.nav_stack.last_mut() {
            current.offset = self.offset;
            current.page = self.page;
        }
    }
}

impl PaginatingView<AppMsg> for CommentState {
    fn jump_page(&self, page: usize) -> AppMsg {
        AppMsg::Comments(CommentMsg::JumpPage(page))
    }

    fn go_back(&self) -> AppMsg {
        AppMsg::Comments(CommentMsg::Back)
    }

    fn go_forward(&self) -> AppMsg {
        AppMsg::Comments(CommentMsg::Forward)
    }

    fn full_count(&self) -> usize {
        self.full_count
    }

    fn current_page(&self) -> usize {
        self.page
    }
}

/// Id for the comment view scroller.
fn comment_scroll_id() -> widget::scrollable::Id {
    widget::scrollable::Id::new("comments")
}
