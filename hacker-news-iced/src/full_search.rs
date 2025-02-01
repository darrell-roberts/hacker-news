use crate::{
    app::AppMsg,
    articles::ArticleMsg,
    comments::CommentMsg,
    common::{error_task, PaginatingView},
    header::HeaderMsg,
    parse_date,
    richtext::render_rich_text,
};
use hacker_news_search::{
    api::{Comment, CommentStack},
    SearchContext, SearchError,
};
use iced::{
    border,
    font::{Style, Weight},
    padding,
    widget::{self, text::Shaping, tooltip::Position},
    Color, Element, Font, Length, Task,
};
use std::sync::{Arc, RwLock};

pub struct FullSearchState {
    pub search: Option<SearchCriteria>,
    pub search_results: Vec<Comment>,
    pub search_context: Arc<RwLock<SearchContext>>,
    pub offset: usize,
    pub page: usize,
    pub full_count: usize,
}

pub enum SearchCriteria {
    Query(String),
    StoryId { story_id: u64, beyond: Option<u64> },
}

#[derive(Debug, Clone)]
pub enum FullSearchMsg {
    Search(String),
    CloseSearch,
    Forward,
    Back,
    ShowThread(u64),
    JumpPage(usize),
    StoryByTime { story_id: u64, beyond: Option<u64> },
    OpenComment { story_id: u64, comment_id: u64 },
}

impl FullSearchState {
    pub fn view(&self) -> iced::Element<AppMsg> {
        let comment_rows = self
            .search_results
            .iter()
            .map(|comment| {
                self.render_comment(comment).style(|theme| {
                    let palette = theme.extended_palette();

                    widget::container::Style {
                        background: Some(palette.background.weak.color.into()),
                        border: border::rounded(8),
                        ..Default::default()
                    }
                })
            })
            .map(iced::Element::from);

        let content = widget::Column::new()
            .push_maybe((self.full_count > 0).then(|| self.pagination_element()))
            .push(
                widget::scrollable(
                    widget::container(widget::Column::with_children(comment_rows).spacing(15))
                        .padding(padding::top(0).bottom(10).left(10).right(25)),
                )
                .id(full_search_scroll_id()),
            )
            .spacing(5);

        widget::container(content).into()
    }

    fn render_comment<'a>(&'a self, comment: &'a Comment) -> widget::Container<'a, AppMsg> {
        let child_comments_button: Element<'_, AppMsg> = if comment.kids.is_empty() {
            widget::text("").into()
        } else {
            widget::button(
                widget::text(format!("💬{}", comment.kids.len())).shaping(Shaping::Advanced),
            )
            .padding(0)
            .on_press(AppMsg::FullSearch(FullSearchMsg::OpenComment {
                story_id: comment.story_id,
                comment_id: comment.id,
            }))
            .style(widget::button::text)
            .into()
        };
        widget::container(
            widget::Column::new()
                .push(
                    widget::Row::new()
                        .push(widget::container(
                            widget::button(widget::text(format!("{}", comment.id)))
                                .on_press(AppMsg::OpenLink {
                                    url: format!(
                                        "https://news.ycombinator.com/item?id={}",
                                        comment.id
                                    ),
                                })
                                .style(widget::button::text)
                                .padding(0),
                        ))
                        .push(
                            widget::container(widget::tooltip(
                                widget::button(widget::text("🧵").shaping(Shaping::Advanced))
                                    .style(widget::button::text)
                                    .on_press(AppMsg::FullSearch(FullSearchMsg::ShowThread(
                                        comment.id,
                                    ))),
                                widget::container(widget::text("Show thread"))
                                    .padding(5)
                                    .style(|_theme| {
                                        widget::container::background(Color::BLACK)
                                            .color(Color::WHITE)
                                            .border(iced::border::rounded(8))
                                    }),
                                Position::Bottom,
                            ))
                            .align_right(Length::Fill),
                        ),
                )
                .push({
                    let s = self
                        .search
                        .iter()
                        .filter_map(|s| match s {
                            SearchCriteria::Query(s) => Some(s.as_str()),
                            SearchCriteria::StoryId { .. } => None,
                        })
                        .next();
                    widget::container(widget::rich_text(render_rich_text(&comment.body, s, false)))
                        .width(Length::FillPortion(6).enclose(Length::Fixed(50.)))
                })
                .push(
                    widget::Row::new()
                        .push(widget::rich_text([
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
                        ]))
                        .push(child_comments_button)
                        .spacing(5),
                )
                .padding([10, 10])
                .spacing(15)
                .width(Length::Fill),
        )
    }

    pub fn update(&mut self, message: FullSearchMsg) -> Task<AppMsg> {
        match message {
            FullSearchMsg::Search(search) => {
                if search.is_empty() {
                    return Task::done(FullSearchMsg::CloseSearch).map(AppMsg::FullSearch);
                } else {
                    if !self.search.iter().any(|s| match s {
                        SearchCriteria::Query(s) => s == &search,
                        SearchCriteria::StoryId { .. } => false,
                    }) {
                        self.page = 1;
                        self.offset = 0;
                    }

                    self.search = Some(SearchCriteria::Query(search.clone()));
                    let g = self.search_context.read().unwrap();
                    match g.search_all_comments(&search, 10, self.offset) {
                        Ok((comments, count)) => {
                            self.search_results = comments;
                            self.full_count = count;
                        }
                        Err(err) => {
                            return error_task(err);
                        }
                    }
                }
                Task::done(AppMsg::CommentsClosed)
            }
            FullSearchMsg::CloseSearch => {
                self.search = None;
                self.offset = 0;
                self.page = 1;
                self.full_count = 0;
                Task::none()
            }
            FullSearchMsg::Forward => {
                self.offset += 10;
                self.page += 1;
                self.paginate_task()
            }
            FullSearchMsg::Back => {
                self.offset -= 10;
                self.page -= 1;

                self.paginate_task()
            }
            FullSearchMsg::ShowThread(comment_id) => {
                let g = self.search_context.read().unwrap();
                match g.parents(comment_id) {
                    Ok(CommentStack { comments, story }) => {
                        let story_id = story.id;
                        Task::batch([
                            Task::done(AppMsg::OpenComment {
                                parent_id: story_id,
                                article: story,
                                comment_stack: comments,
                            }),
                            Task::done(AppMsg::Articles(ArticleMsg::Search(
                                format!("{story_id}",),
                            ))),
                        ])
                    }
                    Err(err) => error_task(err),
                }
            }
            FullSearchMsg::JumpPage(page) => {
                self.page = page;
                if page > 1 {
                    self.offset = 10 * (page - 1);
                } else {
                    self.offset = 0;
                }

                self.paginate_task()
            }
            FullSearchMsg::StoryByTime { story_id, beyond } => {
                self.search = Some(SearchCriteria::StoryId { story_id, beyond });
                match self.search_context.read().unwrap().story_comments_by_date(
                    story_id,
                    beyond,
                    10,
                    self.offset,
                ) {
                    Ok((comments, total_comments)) => {
                        self.search_results = comments;
                        self.full_count = total_comments;
                        Task::none()
                    }
                    Err(err) => error_task(err),
                }
            }
            FullSearchMsg::OpenComment {
                story_id,
                comment_id,
            } => {
                let open_comments_task = || {
                    let g = self.search_context.read().unwrap();
                    let CommentStack { story, comments } = g.parents(comment_id)?;
                    let comment = g.get_comment(comment_id)?;
                    let task = Task::done(AppMsg::OpenComment {
                        article: story,
                        parent_id: comment_id,
                        comment_stack: comments,
                    })
                    .chain(
                        Task::done(CommentMsg::FetchComments {
                            parent_id: comment_id,
                            parent_comment: Some(comment),
                            scroll_to: None,
                        })
                        .map(AppMsg::Comments),
                    );
                    Result::<_, SearchError>::Ok(task)
                };

                match open_comments_task() {
                    Ok(msg) => Task::batch([
                        msg,
                        Task::done(ArticleMsg::Search(format!("{story_id}"))).map(AppMsg::Articles),
                    ]),
                    Err(err) => error_task(err),
                }
            }
        }
    }

    fn paginate_task(&self) -> Task<AppMsg> {
        match self.search.as_ref() {
            Some(s) => match s {
                SearchCriteria::Query(s) => {
                    Task::done(FullSearchMsg::Search(s.to_owned())).map(AppMsg::FullSearch)
                }
                SearchCriteria::StoryId { story_id, beyond } => {
                    Task::done(FullSearchMsg::StoryByTime {
                        story_id: *story_id,
                        beyond: beyond.to_owned(),
                    })
                    .map(AppMsg::FullSearch)
                }
            },
            None => Task::none(),
        }
        .chain(widget::scrollable::scroll_to(
            full_search_scroll_id(),
            Default::default(),
        ))
    }
}

impl PaginatingView<AppMsg> for FullSearchState {
    fn jump_page(&self, page: usize) -> AppMsg {
        AppMsg::FullSearch(FullSearchMsg::JumpPage(page))
    }

    fn go_back(&self) -> AppMsg {
        AppMsg::FullSearch(FullSearchMsg::Back)
    }

    fn go_forward(&self) -> AppMsg {
        AppMsg::FullSearch(FullSearchMsg::Forward)
    }

    fn full_count(&self) -> usize {
        self.full_count
    }

    fn current_page(&self) -> usize {
        self.page
    }
}

fn full_search_scroll_id() -> widget::scrollable::Id {
    widget::scrollable::Id::new("full_search")
}
