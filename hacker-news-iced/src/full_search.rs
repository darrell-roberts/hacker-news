use crate::{app::AppMsg, footer::FooterMsg, parse_date, richtext::render_rich_text};
use hacker_news_search::{api::Comment, SearchContext};
use iced::{
    border,
    font::{Style, Weight},
    padding, widget, Font, Length, Task,
};
use std::sync::Arc;

pub struct FullSearchState {
    pub search: Option<String>,
    pub search_results: Vec<Comment>,
    pub search_context: Arc<SearchContext>,
}

#[derive(Debug, Clone)]
pub enum FullSearchMsg {
    Search(String),
    CloseSearch,
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

        widget::container(widget::scrollable(
            widget::container(widget::Column::with_children(comment_rows).spacing(15))
                .padding(padding::top(0).bottom(10).left(10).right(25)),
        ))
        .into()
    }

    fn render_comment<'a>(&'a self, comment: &'a Comment) -> widget::Container<'a, AppMsg> {
        widget::container(
            widget::Column::new()
                .push(widget::rich_text(render_rich_text(
                    &comment.body,
                    self.search.as_deref(),
                    false,
                )))
                .push(widget::rich_text([
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
                ]))
                .padding([10, 10])
                .spacing(15)
                .width(Length::Fill),
        )
        .into()
    }

    pub fn update(&mut self, message: FullSearchMsg) -> Task<AppMsg> {
        match message {
            FullSearchMsg::Search(search) => {
                if search.is_empty() {
                    self.search = None;
                } else {
                    self.search = Some(search.clone());
                    match self.search_context.search_all_comments(&search, 0) {
                        Ok(comments) => {
                            self.search_results = comments;
                        }
                        Err(err) => {
                            eprintln!("Search failed: {err}");
                            return Task::done(AppMsg::Footer(FooterMsg::Error(err.to_string())));
                        }
                    }
                }
            }
            FullSearchMsg::CloseSearch => self.search = None,
        }
        Task::none()
    }
}
