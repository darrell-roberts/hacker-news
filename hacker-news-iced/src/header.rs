use crate::{app::AppMsg, common::tooltip, footer::FooterMsg, full_search::FullSearchMsg};
use chrono::Local;
use hacker_news_api::ArticleType;
use hacker_news_search::{rebuild_index, IndexStats, RebuildProgress, SearchContext};
use iced::{
    border,
    futures::channel::mpsc,
    widget::{self, button, container, row, text, Column},
    Background, Border, Element, Length, Task,
};
use log::error;
use std::{
    ops::Not,
    sync::{Arc, RwLock},
};

pub struct HeaderState {
    pub search_context: Arc<RwLock<SearchContext>>,
    pub article_count: usize,
    pub article_type: ArticleType,
    pub building_index: bool,
    pub full_search: Option<String>,
}

#[derive(Debug, Clone)]
pub enum HeaderMsg {
    Select {
        article_count: usize,
        article_type: ArticleType,
    },
    ClearVisisted,
    RebuildIndex,
    IndexReady {
        stats: IndexStats,
        category: ArticleType,
    },
    Search(String),
    IndexFailed(String),
    ClearSearch,
}

impl HeaderState {
    pub fn view(&self) -> Element<'_, HeaderMsg> {
        // TODO: Add a search input here that searches the entire index.

        let center_row = container(
            row![
                self.header_type_button(
                    ArticleType::Top,
                    HeaderMsg::Select {
                        article_count: self.article_count,
                        article_type: ArticleType::Top
                    }
                ),
                self.header_type_button(
                    ArticleType::Best,
                    HeaderMsg::Select {
                        article_count: self.article_count,
                        article_type: ArticleType::Best
                    }
                ),
                self.header_type_button(
                    ArticleType::New,
                    HeaderMsg::Select {
                        article_count: self.article_count,
                        article_type: ArticleType::New
                    }
                ),
                self.header_type_button(
                    ArticleType::Ask,
                    HeaderMsg::Select {
                        article_count: self.article_count,
                        article_type: ArticleType::Ask
                    }
                ),
                self.header_type_button(
                    ArticleType::Show,
                    HeaderMsg::Select {
                        article_count: self.article_count,
                        article_type: ArticleType::Show
                    }
                ),
                self.header_type_button(
                    ArticleType::Job,
                    HeaderMsg::Select {
                        article_count: self.article_count,
                        article_type: ArticleType::Job
                    }
                ),
                text(" "),
                self.header_count_button(
                    25,
                    HeaderMsg::Select {
                        article_count: 25,
                        article_type: self.article_type
                    }
                ),
                self.header_count_button(
                    50,
                    HeaderMsg::Select {
                        article_count: 50,
                        article_type: self.article_type
                    }
                ),
                self.header_count_button(
                    75,
                    HeaderMsg::Select {
                        article_count: 75,
                        article_type: self.article_type
                    }
                ),
                // self.header_count_button(
                //     100,
                //     HeaderMsg::Select {
                //         article_count: 100,
                //         article_type: self.article_type
                //     }
                // ),
                // self.header_count_button(
                //     500,
                //     HeaderMsg::Select {
                //         article_count: 500,
                //         article_type: self.article_type
                //     }
                // ),
            ]
            .spacing(10),
        )
        .width(Length::FillPortion(2))
        .padding(5);

        let top_row = widget::container(
            widget::Row::new().push(center_row).push(
                widget::container(
                    widget::Row::new()
                        .push(
                            widget::Row::new()
                                .push(
                                    widget::text_input(
                                        "Search all comments...",
                                        self.full_search.as_deref().unwrap_or_default(),
                                    )
                                    .on_input(HeaderMsg::Search)
                                    .padding(5),
                                )
                                .push(tooltip(
                                    widget::button(
                                        widget::text("⟲").shaping(text::Shaping::Advanced),
                                    )
                                    .on_press(HeaderMsg::ClearSearch),
                                    "Clear search",
                                    widget::tooltip::Position::Bottom,
                                )),
                        )
                        .push(tooltip(
                            widget::button("Re-index")
                                .on_press_maybe(
                                    self.building_index.not().then_some(HeaderMsg::RebuildIndex),
                                )
                                .style(|theme, status| {
                                    let mut style = button::primary(theme, status);
                                    style.border = border::rounded(8.);
                                    style
                                })
                                .padding(5),
                            format!("Re-index {}", self.article_type.as_str()),
                            widget::tooltip::Position::Bottom,
                        ))
                        .push(tooltip(
                            widget::button(widget::text("↻").shaping(text::Shaping::Advanced))
                                .style(|theme, status| {
                                    let mut style = button::primary(theme, status);
                                    style.border = border::rounded(8.);
                                    style
                                })
                                .on_press(HeaderMsg::ClearVisisted)
                                .padding(5),
                            "Clear visited",
                            widget::tooltip::Position::Bottom,
                        ))
                        .spacing(5),
                )
                .padding([5, 5]),
            ),
        )
        .style(|theme| {
            let palette = theme.extended_palette();

            container::Style {
                background: Some(Background::Color(palette.background.strong.color)),
                ..Default::default()
            }
        });

        Column::new().push(top_row).into()
    }

    fn header_type_button(
        &self,
        article_type: ArticleType,
        action: HeaderMsg,
    ) -> Element<'_, HeaderMsg> {
        widget::button(widget::text(article_type.to_string()))
            .on_press(action)
            .style(move |theme, status| {
                let mut style = if self.article_type == article_type {
                    button::primary(theme, status)
                } else {
                    button::secondary(theme, status)
                };

                style.border = Border {
                    radius: 4.into(),
                    ..Default::default()
                };
                style
            })
            .into()
    }

    fn header_count_button(&self, count: usize, action: HeaderMsg) -> Element<'_, HeaderMsg> {
        widget::button(widget::text(count))
            .on_press(action)
            .style(move |theme, status| {
                let mut style = if self.article_count == count {
                    button::primary(theme, status)
                } else {
                    button::secondary(theme, status)
                };

                style.border = Border {
                    radius: 4.into(),
                    ..Default::default()
                };
                style
            })
            .into()
    }

    pub fn update(&mut self, message: HeaderMsg) -> Task<AppMsg> {
        match message {
            HeaderMsg::Select {
                article_count,
                article_type,
            } => {
                self.article_type = article_type;
                self.article_count = article_count;
                Task::batch([
                    Task::done(HeaderMsg::ClearSearch).map(AppMsg::Header),
                    Task::done(AppMsg::SwitchIndex {
                        category: self.article_type,
                        count: article_count,
                    }),
                ])
            }
            HeaderMsg::ClearVisisted => Task::done(AppMsg::ClearVisited),
            HeaderMsg::RebuildIndex => {
                self.building_index = true;
                let s = self.search_context.clone();
                let category = self.article_type;

                let (tx, rx) = mpsc::channel::<RebuildProgress>(100);
                let fut = rebuild_index(s, category, tx);

                Task::batch([
                    Task::perform(fut, move |result| match result {
                        Ok(stats) => AppMsg::Header(HeaderMsg::IndexReady { stats, category }),
                        Err(err) => {
                            error!("Failed to create index {err}");
                            AppMsg::Header(HeaderMsg::IndexFailed(err.to_string()))
                        }
                    }),
                    Task::done(FooterMsg::Error("Building index...".into())).map(AppMsg::Footer),
                    Task::run(rx, FooterMsg::IndexProgress).map(AppMsg::Footer),
                ])
            }
            HeaderMsg::IndexReady { stats, category } => {
                self.building_index = false;
                self.article_type = category;
                Task::done(AppMsg::SwitchIndex {
                    category,
                    count: self.article_count,
                })
                .chain(Task::batch([
                    Task::done(FooterMsg::LastUpdate(Local::now())).map(AppMsg::Footer),
                    Task::done(FooterMsg::IndexStats { stats, category }).map(AppMsg::Footer),
                ]))
            }
            HeaderMsg::IndexFailed(err) => {
                self.building_index = false;
                Task::done(FooterMsg::Error(err)).map(AppMsg::Footer)
            }
            HeaderMsg::Search(search) => {
                if search.is_empty() {
                    Task::done(HeaderMsg::ClearSearch).map(AppMsg::Header)
                } else {
                    self.full_search = Some(search.clone());
                    Task::done(FullSearchMsg::Search(search)).map(AppMsg::FullSearch)
                }
            }
            HeaderMsg::ClearSearch => {
                self.full_search = None;
                Task::done(AppMsg::FullSearch(FullSearchMsg::CloseSearch))
            }
        }
    }
}
