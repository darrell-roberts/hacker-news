//! State and view for viewing comments.
use crate::{
    app::AppMsg,
    articles::ArticleMsg,
    common::{self, error_task, FontExt as _, PaginatingView},
    full_search::FullSearchMsg,
    header::HeaderMsg,
    parse_date,
    richtext::render_rich_text,
    ROBOTO_FONT,
};
use hacker_news_search::{
    api::{Comment, Story},
    SearchContext,
};
use iced::{
    border, padding,
    widget::{
        self, button, container, scrollable::AbsoluteOffset, text::Shaping, Column, Container,
    },
    Border, Color, Element, Length, Shadow, Task,
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

impl NavStack {
    /// Create a root stack node.
    pub fn root() -> Self {
        Self {
            comment: None,
            offset: 0,
            page: 1,
            scroll_offset: None,
        }
    }
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
    /// Parent id being viewed.
    pub parent_id: u64,
    /// Active comment
    pub active_comment_id: Option<u64>,
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
    // OpenSearch,
    CloseSearch,
    Oneline,
    Forward,
    Back,
    JumpPage(usize),
    Close(u64),
    ScrollOffset(AbsoluteOffset),
    Activate(u64),
    ShowThread(u64),
}

impl CommentState {
    /// Render the comments
    pub fn view(&self) -> Element<'_, AppMsg> {
        let article_text = self
            .article
            .body
            .as_deref()
            .map(|text| {
                widget::rich_text(render_rich_text(text, self.search.as_deref(), false))
                    .on_link_click(|url| AppMsg::OpenLink { url })
            })
            .map(|rt| container(rt).padding([10, 10]).into());

        let total_parents = self
            .nav_stack
            .iter()
            .filter_map(|stack| stack.comment.as_ref())
            .count();

        let parent_comments = self
            .nav_stack
            .iter()
            .filter_map(|stack| stack.comment.as_ref())
            .zip(1..)
            .map(|(parent, index)| {
                widget::Row::with_children((1..=index).map(|current| {
                    if current == 1 {
                        // Nothing.
                        widget::container("").into()
                    } else if current == index {
                        // Show connector
                        common::thread_pointer()
                    } else {
                        // Indent
                        widget::container("").width(Length::Fixed(20.)).into()
                    }
                }))
                .push(self.render_comment(parent, true).style(|theme| {
                    let palette = theme.extended_palette();

                    container::Style {
                        border: Border {
                            color: palette.secondary.weak.color,
                            width: 1.,
                            radius: 8.into(),
                        },
                        shadow: Shadow {
                            color: Color::BLACK,
                            offset: iced::Vector { x: 2., y: 2. },
                            blur_radius: 5.,
                        },
                        ..Default::default()
                    }
                }))
            })
            .map(Element::from)
            .collect::<Vec<_>>();

        let comment_rows = self
            .comments
            .iter()
            .map(|item| {
                let comment_area = self.render_comment(item, false).style(|theme| {
                    let palette = theme.extended_palette();
                    container::Style {
                        background: Some(if self.active_comment_id == Some(item.id) {
                            palette.background.strong.color.into()
                        } else {
                            palette.background.weak.color.into()
                        }),
                        border: border::rounded(8),
                        shadow: Shadow {
                            color: Color::BLACK,
                            offset: iced::Vector { x: 2., y: 2. },
                            blur_radius: 5.,
                        },
                        ..Default::default()
                    }
                });

                Element::from(
                    widget::Row::with_children((1..=parent_comments.len()).map(|current| {
                        if current == total_parents {
                            // Show connector
                            common::thread_pointer()
                        } else {
                            // Indent
                            widget::container("").width(Length::Fixed(20.)).into()
                        }
                    }))
                    .push(comment_area),
                )
            })
            .collect::<Vec<_>>();

        let content = Column::new()
            .push(
                widget::Row::new()
                    .push(
                        widget::Row::new()
                            .push(
                                widget::text_input(
                                    "Search within story...",
                                    self.search.as_deref().unwrap_or_default(),
                                )
                                .id(widget::Id::new("comment_search"))
                                .on_input(|input| AppMsg::Comments(CommentMsg::Search(input))),
                            )
                            .push(common::tooltip(
                                widget::button(widget::text("⟲").shaping(Shaping::Advanced))
                                    .on_press(AppMsg::Comments(CommentMsg::CloseSearch)),
                                "Clear search",
                                widget::tooltip::Position::FollowCursor,
                            )),
                    )
                    .push(widget::text(format!("{}", self.full_count)))
                    .push(
                        widget::toggler(self.oneline)
                            .label("oneline")
                            .on_toggle(|_| AppMsg::Comments(CommentMsg::Oneline)),
                    )
                    .push(common::tooltip(
                        widget::button(widget::text("⌛").shaping(Shaping::Advanced)).on_press(
                            AppMsg::FullSearch(FullSearchMsg::StoryByTime {
                                story_id: self.article.id,
                                beyond: None,
                            }),
                        ),
                        "Sorted by latest",
                        widget::tooltip::Position::Bottom,
                    ))
                    .spacing(5),
            )
            .push(
                widget::scrollable(
                    widget::Column::new()
                        .push(
                            self.search
                                .is_none()
                                .then(|| Column::with_children(article_text).spacing(15)),
                        )
                        .push(
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
            )
            .push((self.full_count > 10).then(|| self.pagination_element()))
            .padding(iced::padding::top(5));

        container(content.width(Length::Fill)).into()
    }

    /// Render a single comment
    fn render_comment<'a>(
        &'a self,
        comment: &'a Comment,
        is_parent: bool,
    ) -> Container<'a, AppMsg> {
        let child_comments_button: Element<'_, AppMsg> = if comment.kids.is_empty() {
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
            widget::mouse_area(
                container(
                    widget::Column::new()
                        .push(is_parent.then(|| {
                            widget::container(
                                widget::button(widget::text("X").size(10))
                                    .on_press(AppMsg::Comments(CommentMsg::Close(comment.id))),
                            )
                            .align_right(Length::Fill)
                        }))
                        .push(self.search.is_some().then(|| {
                            widget::container(widget::tooltip(
                                widget::button(widget::text("🧵").shaping(Shaping::Advanced))
                                    .style(widget::button::text)
                                    .on_press(AppMsg::Comments(CommentMsg::ShowThread(comment.id))),
                                widget::container(widget::text("Show thread"))
                                    .padding(5)
                                    .style(|_theme| {
                                        widget::container::background(Color::BLACK)
                                            .color(Color::WHITE)
                                            .border(iced::border::rounded(8))
                                    }),
                                widget::tooltip::Position::Bottom,
                            ))
                            .align_right(Length::Fill)
                        }))
                        .push(
                            widget::rich_text(render_rich_text(
                                &comment.body,
                                self.search.as_deref(),
                                self.oneline,
                            ))
                            .on_link_click(|url| AppMsg::OpenLink { url }),
                        )
                        .push(
                            widget::row![
                                widget::rich_text::<'_, AppMsg, AppMsg, _, _>([
                                    widget::span(format!("by {}", comment.by))
                                        .link(AppMsg::Header(HeaderMsg::Search(format!(
                                            "by:{}",
                                            comment.by
                                        ))))
                                        .font(ROBOTO_FONT.italic())
                                        .size(14),
                                    widget::span(" "),
                                    widget::span(parse_date(comment.time).unwrap_or_default())
                                        .font(ROBOTO_FONT.italic().weight_light())
                                        .size(10),
                                ]),
                                child_comments_button,
                                widget::container(common::tooltip(
                                    widget::button(widget::text(format!("{}", comment.id)))
                                        .on_press(AppMsg::OpenLink {
                                            url: format!(
                                                "https://news.ycombinator.com/item?id={}",
                                                comment.id
                                            ),
                                        })
                                        .style(widget::button::text)
                                        .padding(0),
                                    "Open in browser",
                                    widget::tooltip::Position::Left
                                ))
                                .align_right(Length::Fill)
                            ]
                            .spacing(5),
                        )
                        .padding(10)
                        .spacing(15)
                        .width(Length::Fill),
                )
                .clip(false),
            )
            .on_press(AppMsg::Comments(CommentMsg::Activate(comment.id))),
        )
    }

    /// Update comment viewing state.
    pub fn update(&mut self, message: CommentMsg) -> Task<AppMsg> {
        match message {
            CommentMsg::FetchComments {
                parent_id,
                parent_comment,
                scroll_to,
            } => {
                self.search = None;
                if let Some(parent) = parent_comment {
                    // We are viewing a nested comment
                    if self.parent_id != parent.id {
                        if let Some(index) = self.nav_stack.iter().enumerate().find_map(
                            |(index, stack_item)| {
                                matches!(stack_item.comment.as_ref(), Some(c) if c.id == parent_id)
                                    .then_some(index)
                            },
                        ) {
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

                        Task::batch([
                            widget::operation::scroll_to(
                                comment_scroll_id(),
                                scroll_to.unwrap_or_default(),
                            ),
                            Task::done(ArticleMsg::ViewingItem(self.article.id))
                                .map(AppMsg::Articles),
                        ])
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
                if let Some(c) = self.nav_stack.pop().and_then(|stack| stack.comment) {
                    self.active_comment_id.replace(c.id);
                };

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
            // CommentMsg::OpenSearch => {
            //     self.search = Some(String::new());
            //     self.nav_stack.push(NavStack {
            //         comment: None,
            //         offset: 0,
            //         page: 1,
            //         scroll_offset: None,
            //     });
            //     self.offset = 0;
            //     self.page = 1;
            //     self.comments = Vec::new();
            //     widget::text_input::focus(widget::text_input::Id::new("comment_search"))
            // }
            CommentMsg::CloseSearch => {
                self.search = None;

                // Task::done(CommentMsg::PopNavStack).map(AppMsg::Comments)
                Task::done(CommentMsg::JumpPage(1)).map(AppMsg::Comments)
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
                self.active_comment_id.replace(comment_id);

                // Check if this is a top level comment.
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
            CommentMsg::Activate(comment_id) => {
                self.active_comment_id = Some(comment_id);
                Task::none()
            }
            CommentMsg::ShowThread(comment_id) => {
                common::show_thread(self.search_context.clone(), comment_id)
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
        .chain(widget::operation::scroll_to(
            comment_scroll_id(),
            widget::operation::AbsoluteOffset { x: 0.0, y: 0.0 },
        ))
    }

    fn update_nav_stack(&mut self) {
        if let Some(current) = self.nav_stack.last_mut() {
            current.offset = self.offset;
            current.page = self.page;
            current.scroll_offset = None;
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
fn comment_scroll_id() -> widget::Id {
    widget::Id::new("comments")
}
