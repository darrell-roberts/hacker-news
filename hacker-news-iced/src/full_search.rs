use crate::{
    app::AppMsg, articles::ArticleMsg, footer::FooterMsg, parse_date, richtext::render_rich_text,
};
use hacker_news_search::{
    api::{Comment, CommentStack},
    SearchContext,
};
use iced::{
    alignment::Vertical,
    border,
    font::{Style, Weight},
    padding,
    widget::{self, text::Shaping},
    Font, Length, Task, Theme,
};
use log::error;
use std::sync::{Arc, RwLock};

pub struct FullSearchState {
    pub search: Option<String>,
    pub search_results: Vec<Comment>,
    pub search_context: Arc<RwLock<SearchContext>>,
    pub offset: usize,
    pub page: usize,
    pub full_count: usize,
}

#[derive(Debug, Clone)]
pub enum FullSearchMsg {
    Search(String),
    CloseSearch,
    Forward,
    Back,
    ShowThread(u64),
    JumpPage(usize),
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

        let pagination = || {
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
                .on_press(AppMsg::FullSearch(FullSearchMsg::JumpPage(page)))
                .into()
            });

            widget::container(
                widget::Row::new()
                    .push(
                        widget::button(widget::text("←").shaping(Shaping::Advanced))
                            .on_press_maybe(
                                self.page
                                    .gt(&1)
                                    .then_some(AppMsg::FullSearch(FullSearchMsg::Back)),
                            ),
                    )
                    .extend(pages)
                    .push(
                        widget::button(widget::text("→").shaping(Shaping::Advanced))
                            .on_press_maybe(
                                (self.page < (self.full_count / 10) + 1)
                                    .then_some(AppMsg::FullSearch(FullSearchMsg::Forward)),
                            ),
                    )
                    .spacing(2)
                    .align_y(Vertical::Center)
                    .wrap(),
            )
            .center_x(Length::Fill)
        };

        let content = widget::Column::new()
            .push(
                widget::container(widget::text(format!("{}", self.full_count)))
                    .align_right(Length::Fill)
                    .padding(iced::padding::right(5)),
            )
            .push_maybe((self.full_count > 0).then(pagination))
            .push(
                widget::scrollable(
                    widget::container(widget::Column::with_children(comment_rows).spacing(15))
                        .padding(padding::top(0).bottom(10).left(10).right(25)),
                )
                .id(widget::scrollable::Id::new("full_search")),
            )
            .spacing(5);

        widget::container(content).into()
    }

    fn render_comment<'a>(&'a self, comment: &'a Comment) -> widget::Container<'a, AppMsg> {
        widget::container(
            widget::button(
                widget::Column::new()
                    .push(widget::rich_text(render_rich_text(
                        &comment.body,
                        self.search.as_deref(),
                        false,
                    )))
                    .push(
                        widget::Row::new()
                            .push(widget::rich_text([
                                widget::span(format!(" by {}", comment.by))
                                    .link(AppMsg::FullSearch(FullSearchMsg::Search(format!(
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
                            .push(
                                widget::container(
                                    widget::button(widget::text(format!("{}", comment.id)))
                                        .on_press(AppMsg::OpenLink {
                                            url: format!(
                                                "https://news.ycombinator.com/item?id={}",
                                                comment.id
                                            ),
                                            item_id: comment.story_id,
                                        })
                                        .style(widget::button::text)
                                        .padding(0),
                                )
                                .align_right(Length::Fill),
                            ),
                    )
                    .padding([10, 10])
                    .spacing(15)
                    .width(Length::Fill),
            )
            .style(|theme, _status| widget::button::text(theme, widget::button::Status::Active))
            .on_press(AppMsg::FullSearch(FullSearchMsg::ShowThread(comment.id))),
        )
    }

    pub fn update(&mut self, message: FullSearchMsg) -> Task<AppMsg> {
        match message {
            FullSearchMsg::Search(search) => {
                if search.is_empty() {
                    return Task::done(FullSearchMsg::CloseSearch).map(AppMsg::FullSearch);
                } else {
                    if self.search.as_deref().unwrap_or_default() != search {
                        self.page = 1;
                        self.offset = 0;
                    }
                    self.search = Some(search.clone());
                    let g = self.search_context.read().unwrap();
                    match g.search_all_comments(&search, 10, self.offset) {
                        Ok((comments, count)) => {
                            self.search_results = comments;
                            self.full_count = count;
                        }
                        Err(err) => {
                            error!("Search failed: {err}");
                            return Task::done(AppMsg::Footer(FooterMsg::Error(err.to_string())));
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
                Task::done(FullSearchMsg::Search(
                    self.search.as_deref().unwrap_or_default().to_owned(),
                ))
                .map(AppMsg::FullSearch)
            }
            FullSearchMsg::Back => {
                self.offset -= 10;
                self.page -= 1;

                Task::done(FullSearchMsg::Search(
                    self.search.as_deref().unwrap_or_default().to_owned(),
                ))
                .map(AppMsg::FullSearch)
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
                    Err(err) => Task::done(FooterMsg::Error(err.to_string())).map(AppMsg::Footer),
                }
            }
            FullSearchMsg::JumpPage(page) => {
                self.page = page;
                if page > 1 {
                    self.offset = 10 * (page - 1);
                } else {
                    self.offset = 0;
                }

                Task::done(FullSearchMsg::Search(
                    self.search.as_deref().unwrap_or_default().to_owned(),
                ))
                .map(AppMsg::FullSearch)
            }
        }
    }
}
