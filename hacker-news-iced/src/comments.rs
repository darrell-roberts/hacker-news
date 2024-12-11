use crate::{
    app::AppMsg, footer::FooterMsg, full_search::FullSearchMsg, parse_date,
    richtext::render_rich_text,
};
use hacker_news_search::{
    api::{Comment, Story},
    SearchContext,
};
use iced::{
    alignment::Vertical,
    border,
    font::{Style, Weight},
    padding,
    widget::{self, button, container, text::Shaping, Column, Container},
    Border, Element, Font, Length, Task, Theme,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct NavStack {
    pub comment: Option<Comment>,
    pub offset: usize,
    pub page: usize,
}

/// Comment state
pub struct CommentState {
    pub search_context: Arc<SearchContext>,
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
    },
    PopNavStack,
    Search(String),
    OpenSearch,
    CloseSearch,
    Oneline,
    Forward,
    Back,
    JumpPage(usize),
}

impl CommentState {
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
                .id(widget::scrollable::Id::new("comments"))
                .height(Length::Fill),
            );

        container(content.width(Length::Fill)).into()
    }

    fn pagination_element(&self) -> Element<AppMsg> {
        let (div, rem) = (self.full_count / 10, self.full_count % 10);
        let max = if rem > 0 { div + 1 } else { div };
        let pages = (1..=max).map(|page| {
            widget::button(
                widget::container(widget::text(format!("{page}")))
                    .style(move |theme: &Theme| {
                        let palette = theme.extended_palette();
                        if page == self.page {
                            widget::container::rounded_box(theme)
                                .background(palette.secondary.strong.color)
                        } else {
                            widget::container::transparent(theme)
                        }
                    })
                    .padding(5),
            )
            .style(widget::button::text)
            .padding(0)
            .on_press(AppMsg::Comments(CommentMsg::JumpPage(page)))
            .into()
        });

        widget::container(
            widget::Row::new()
                .push(
                    widget::button(widget::text("🡸").shaping(Shaping::Advanced)).on_press_maybe(
                        self.page
                            .gt(&1)
                            .then_some(AppMsg::Comments(CommentMsg::Back)),
                    ),
                )
                .extend(pages)
                .push(
                    widget::button(widget::text("🡺").shaping(Shaping::Advanced)).on_press_maybe(
                        (self.page < (self.full_count / 10) + 1)
                            .then_some(AppMsg::Comments(CommentMsg::Forward)),
                    ),
                )
                .spacing(2)
                .align_y(Vertical::Center),
        )
        .center_x(Length::Fill)
        .padding([5, 0])
        .into()
    }

    fn render_comment<'a>(
        &'a self,
        comment: &'a Comment,
        is_parent: bool,
    ) -> Container<'a, AppMsg> {
        let by_button: Element<'_, AppMsg> = if comment.kids.is_empty() {
            widget::text("").into()
        } else if is_parent {
            widget::text(format!("💬{}", comment.kids.len()))
                .shaping(Shaping::Advanced)
                .into()
        } else {
            button(widget::text(format!("💬{}", comment.kids.len())).shaping(Shaping::Advanced))
                .padding(0)
                .on_press(AppMsg::Comments(CommentMsg::FetchComments {
                    parent_id: comment.id,
                    parent_comment: Some(comment.clone()),
                }))
                .style(button::text)
                .into()
        };

        container(
            widget::column![
                widget::rich_text(render_rich_text(
                    &comment.body,
                    self.search.as_deref(),
                    self.oneline
                )),
                widget::row![
                    widget::rich_text([
                        widget::span(format!(" by {}", comment.by))
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
                ]
                .spacing(5)
            ]
            .padding([10, 10])
            .spacing(15)
            .width(Length::Fill),
        )
        .clip(false)
    }

    pub fn update(&mut self, message: CommentMsg) -> Task<AppMsg> {
        match message {
            CommentMsg::FetchComments {
                parent_id,
                parent_comment,
            } => {
                if let Some(parent) = parent_comment {
                    // We are viewing a nested comment
                    if self.parent_id != parent.id {
                        self.nav_stack.push(NavStack {
                            comment: Some(parent),
                            offset: 0,
                            page: 1,
                        });
                        self.offset = 0;
                        self.page = 1;
                    }
                }
                let fetch_task = match self.search_context.comments(parent_id, 10, self.offset) {
                    Ok((comments, full_count)) => {
                        self.full_count = full_count;
                        self.comments = comments;

                        Task::none()
                    }
                    Err(err) => Task::done(FooterMsg::Error(err.to_string())).map(AppMsg::Footer),
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

                    match self.search_context.search_comments(
                        &search,
                        self.article.id,
                        10,
                        self.offset,
                    ) {
                        Ok((comments, count)) => {
                            self.comments = comments;
                            self.full_count = count;
                            Task::none()
                        }
                        Err(err) => {
                            eprintln!("Failed search: {err}");
                            Task::done(AppMsg::Footer(FooterMsg::Error(err.to_string())))
                        }
                    }
                }
            }
            CommentMsg::OpenSearch => {
                self.search = Some(String::new());
                self.nav_stack.push(NavStack {
                    comment: None,
                    offset: 0,
                    page: 1,
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
        }
    }

    fn paginate_task(&self) -> Task<AppMsg> {
        match self.search.as_ref() {
            Some(s) => Task::done(CommentMsg::Search(s.to_owned())).map(AppMsg::Comments),
            None => Task::done(CommentMsg::FetchComments {
                parent_id: self.parent_id,
                parent_comment: None,
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
