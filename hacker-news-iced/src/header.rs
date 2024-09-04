use crate::app::{App, AppMsg};
use hacker_news_api::ArticleType;
use iced::{
    widget::{self, button, container, row, text, text_input::Id, Column},
    Background, Border, Element, Length,
};

impl App {
    pub fn render_header(&self) -> Element<'_, AppMsg> {
        let top_row = container(
            row![
                self.header_type_button(
                    ArticleType::Top,
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::Top
                    }
                ),
                self.header_type_button(
                    ArticleType::Best,
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::Best
                    }
                ),
                self.header_type_button(
                    ArticleType::New,
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::New
                    }
                ),
                self.header_type_button(
                    ArticleType::Ask,
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::Ask
                    }
                ),
                self.header_type_button(
                    ArticleType::Show,
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::Show
                    }
                ),
                self.header_type_button(
                    ArticleType::Job,
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::Job
                    }
                ),
                text(" "),
                self.header_count_button(
                    25,
                    AppMsg::Fetch {
                        limit: 25,
                        article_type: self.showing.article_type
                    }
                ),
                self.header_count_button(
                    50,
                    AppMsg::Fetch {
                        limit: 50,
                        article_type: self.showing.article_type
                    }
                ),
                self.header_count_button(
                    75,
                    AppMsg::Fetch {
                        limit: 75,
                        article_type: self.showing.article_type
                    }
                ),
                self.header_count_button(
                    100,
                    AppMsg::Fetch {
                        limit: 100,
                        article_type: self.showing.article_type
                    }
                ),
                self.header_count_button(
                    500,
                    AppMsg::Fetch {
                        limit: 500,
                        article_type: self.showing.article_type
                    }
                ),
            ]
            .spacing(10),
        )
        .style(|theme| {
            let palette = theme.extended_palette();

            container::Style {
                background: Some(Background::Color(palette.background.strong.color)),
                ..Default::default()
            }
        })
        .center_x(1)
        .width(Length::Fill)
        .padding([5, 0]);

        Column::new()
            .push(top_row)
            .push_maybe(self.search.as_ref().map(|search| {
                widget::text_input("Search...", search)
                    .id(Id::new("search"))
                    .on_input(AppMsg::Search)
            }))
            .into()
    }

    fn header_type_button(&self, article_type: ArticleType, action: AppMsg) -> Element<'_, AppMsg> {
        widget::button(widget::text(article_type.to_string()))
            .on_press(action)
            .style(move |theme, status| {
                let mut style = if self.showing.article_type == article_type {
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

    fn header_count_button(&self, count: usize, action: AppMsg) -> Element<'_, AppMsg> {
        widget::button(widget::text(count))
            .on_press(action)
            .style(move |theme, status| {
                let mut style = if self.showing.limit == count {
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
}
