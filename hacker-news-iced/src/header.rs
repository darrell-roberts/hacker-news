use crate::{app::AppMsg, articles::ArticleMsg};
use hacker_news_api::ArticleType;
use iced::{
    widget::{self, button, container, row, text, text_input::Id, Column},
    Background, Border, Element, Length, Task,
};

pub struct HeaderState {
    pub article_count: usize,
    pub article_type: ArticleType,
    pub search: Option<String>,
}

#[derive(Debug, Clone)]
pub enum HeaderMsg {
    // OpenSearch,
    // CloseSearch,
    Select {
        article_count: usize,
        article_type: ArticleType,
    },
    // Search(String),
    ClearVisisted,
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
            widget::Row::new().push(center_row).push(
                widget::container(widget::tooltip(
                    widget::button(widget::text("↻").shaping(text::Shaping::Advanced))
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

        Column::new()
            .push(top_row)
            // .push_maybe(self.search.as_ref().map(|search| {
            //     widget::text_input("Search...", search)
            //         .id(Id::new("search"))
            //         .on_input(HeaderMsg::Search)
            // }))
            .into()
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
            // HeaderMsg::OpenSearch => {
            //     self.search = Some(String::new());
            //     widget::text_input::focus(widget::text_input::Id::new("search"))
            // }
            // HeaderMsg::CloseSearch => {
            //     self.search = None;
            //     Task::done(ArticleMsg::Search(String::new())).map(AppMsg::Articles)
            // }
            HeaderMsg::Select {
                article_count,
                article_type,
            } => {
                self.article_type = article_type;
                self.article_count = article_count;
                Task::done(ArticleMsg::Fetch {
                    limit: article_count,
                    article_type,
                })
                .map(AppMsg::Articles)
            }
            // HeaderMsg::Search(search) => {
            //     self.search = Some(search.clone());
            //     Task::done(ArticleMsg::Search(search)).map(AppMsg::Articles)
            // }
            HeaderMsg::ClearVisisted => Task::done(AppMsg::ClearVisited),
        }
    }
}
