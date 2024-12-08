use crate::{app::AppMsg, footer::FooterMsg};
use hacker_news_api::ArticleType;
use hacker_news_search::{rebuild_index, SearchContext};
use iced::{
    border,
    widget::{self, button, container, row, text, Column},
    Background, Border, Element, Length, Task,
};
use std::{ops::Not, sync::Arc};

pub struct HeaderState {
    pub search_context: Arc<SearchContext>,
    pub article_count: usize,
    pub article_type: ArticleType,
    pub building_index: bool,
}

#[derive(Debug, Clone)]
pub enum HeaderMsg {
    Select {
        article_count: usize,
        article_type: ArticleType,
    },
    ClearVisisted,
    RebuildIndex,
    IndexReady,
}

impl HeaderState {
    pub fn view(&self) -> Element<'_, HeaderMsg> {
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
                self.header_count_button(
                    100,
                    HeaderMsg::Select {
                        article_count: 100,
                        article_type: self.article_type
                    }
                ),
                self.header_count_button(
                    500,
                    HeaderMsg::Select {
                        article_count: 500,
                        article_type: self.article_type
                    }
                ),
            ]
            .spacing(10),
        )
        .center_x(1)
        .width(Length::Fill)
        .padding([5, 0]);

        let top_row = widget::container(
            widget::Row::new()
                .push(center_row)
                .push(
                    widget::container(
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
                    )
                    .padding([5, 5]),
                )
                .push(
                    widget::container(widget::tooltip(
                        widget::button(widget::text("↻").shaping(text::Shaping::Advanced))
                            .style(|theme, status| {
                                let mut style = button::primary(theme, status);
                                style.border = border::rounded(8.);
                                style
                            })
                            .on_press(HeaderMsg::ClearVisisted)
                            .padding(5),
                        widget::container(widget::text("Clear visited").color(iced::Color::WHITE))
                            .style(|_| {
                                widget::container::Style::default()
                                    .background(Background::Color(iced::Color::BLACK))
                            })
                            .padding([2, 2]),
                        widget::tooltip::Position::Left,
                    ))
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
                // Task::done(ArticleMsg::Fetch {
                //     limit: article_count,
                //     article_type,
                // })
                // .map(AppMsg::Articles)
                Task::none()
            }
            HeaderMsg::ClearVisisted => Task::done(AppMsg::ClearVisited),
            HeaderMsg::RebuildIndex => {
                self.building_index = true;
                let s = self.search_context.clone();
                Task::batch([
                    Task::perform(
                        async move { rebuild_index(&s).await },
                        |result| match result {
                            Ok(_) => AppMsg::IndexReady,
                            Err(err) => {
                                eprintln!("Failed to create index {err}");
                                AppMsg::Footer(FooterMsg::Error(err.to_string()))
                            }
                        },
                    ),
                    Task::done(AppMsg::Footer(FooterMsg::Error("Building index...".into()))),
                ])
            }
            HeaderMsg::IndexReady => {
                self.building_index = false;
                Task::none()
            }
        }
    }
}
