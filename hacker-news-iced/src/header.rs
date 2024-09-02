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
                header_button(
                    "Top",
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::Top
                    }
                ),
                header_button(
                    "Best",
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::Best
                    }
                ),
                header_button(
                    "New",
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::New
                    }
                ),
                text("|"),
                // Rule::vertical(1),
                header_button(
                    "Ask",
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::Ask
                    }
                ),
                header_button(
                    "Show",
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::Show
                    }
                ),
                header_button(
                    "Job",
                    AppMsg::Fetch {
                        limit: self.showing.limit,
                        article_type: ArticleType::Job
                    }
                ),
                text("|"),
                // Rule::vertical(1),
                header_button(
                    "25",
                    AppMsg::Fetch {
                        limit: 25,
                        article_type: self.showing.article_type
                    }
                ),
                header_button(
                    "50",
                    AppMsg::Fetch {
                        limit: 50,
                        article_type: self.showing.article_type
                    }
                ),
                header_button(
                    "75",
                    AppMsg::Fetch {
                        limit: 75,
                        article_type: self.showing.article_type
                    }
                ),
                header_button(
                    "100",
                    AppMsg::Fetch {
                        limit: 100,
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
}

fn header_button(label: &str, action: AppMsg) -> Element<'_, AppMsg> {
    widget::button(label)
        .on_press(action)
        .style(|theme, status| {
            let mut style = button::primary(theme, status);

            style.border = Border {
                radius: 4.into(),
                ..Default::default()
            };
            style
        })
        .into()
}
