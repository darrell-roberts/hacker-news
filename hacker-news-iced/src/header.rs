use hacker_news_api::ArticleType;
use iced::{
    theme::Button,
    widget::{button, container, row, text},
    Border, Element, Length, Theme,
};

use crate::app::{App, AppMsg};

impl App {
    pub fn render_header(&self) -> Element<'_, AppMsg> {
        container(
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
                // Rule::horizontal(0.1),
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
                // Rule::horizontal(0.1),
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
        .center_x()
        .width(Length::Fill)
        .into()
    }
}

fn header_button(label: &str, action: AppMsg) -> Element<'_, AppMsg> {
    button(label)
        .on_press(action)
        .style(Button::Custom(Box::new(HeaderButtonStyle)))
        .into()
}

pub struct HeaderButtonStyle;

impl button::StyleSheet for HeaderButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let button = Button::Primary;
        let appearance = style.active(&button);

        button::Appearance {
            border: Border {
                radius: 4.into(),
                ..Default::default()
            },
            ..appearance
        }
    }
}
