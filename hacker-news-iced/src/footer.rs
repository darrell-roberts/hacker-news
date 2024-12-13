use std::collections::HashMap;

use crate::app::AppMsg;
use chrono::{DateTime, Local, Utc};
use chrono_tz::America::New_York;
use hacker_news_api::ArticleType;
use hacker_news_search::IndexStats;
use iced::{
    alignment::Vertical,
    font::{Style, Weight},
    widget::{container, pick_list, text, Row},
    Background, Element, Font, Length, Task, Theme,
};

pub struct FooterState {
    pub status_line: String,
    pub last_update: Option<DateTime<Local>>,
    pub scale: f64,
    pub current_index_stats: Option<IndexStats>,
    pub index_stats: HashMap<&'static str, IndexStats>,
}

#[derive(Debug, Clone)]
pub enum FooterMsg {
    Error(String),
    LastUpdate(DateTime<Local>),
    Url(String),
    NoUrl,
    Scale(f64),
    IndexStats {
        stats: IndexStats,
        category: ArticleType,
    },
    CurrentIndex(ArticleType),
}

impl FooterState {
    pub fn view<'a>(&'a self, theme: &'a Theme) -> Element<'a, AppMsg> {
        let themes = Theme::ALL;

        let light_font = || Font {
            style: Style::Italic,
            weight: Weight::Light,
            ..Default::default()
        };

        let row = Row::new()
            .push(
                text(&self.status_line)
                    .font(light_font())
                    .width(Length::Fill.enclose(Length::Fill))
                    .align_y(Vertical::Bottom),
            )
            .push(
                container(
                    Row::new()
                        .push_maybe(self.current_index_stats.as_ref().map(|stats| {
                            Row::new()
                                .push(text(format!("{}", stats.category)))
                                .push_maybe(
                                    DateTime::<Utc>::from_timestamp(stats.built_on as i64, 0)
                                        .map(|dt| dt.with_timezone(&New_York))
                                        .map(|dt| text(dt.format("%d/%m/%y %H:%M,").to_string())),
                                )
                                .push_maybe((stats.build_time.as_secs() / 60 != 0).then(|| {
                                    text(format!("{} min", stats.build_time.as_secs() / 60))
                                }))
                                .push_maybe(
                                    (stats.build_time.as_millis() < 1000).then(|| {
                                        text(format!("{} ms", stats.build_time.as_millis()))
                                    }),
                                )
                                .push_maybe((stats.build_time.as_secs() >= 1).then(|| {
                                    text(format!("{} secs,", stats.build_time.as_secs() % 60))
                                }))
                                .push(text(format!(
                                    "docs: {}, comments: {}, stories: {}, jobs: {}, polls: {}",
                                    stats.total_documents,
                                    stats.total_comments,
                                    stats.total_stories,
                                    stats.total_jobs,
                                    stats.total_polls
                                )))
                                .spacing(5)
                        }))
                        .push(text(format!("Scale: {:.2}", self.scale)).font(light_font()))
                        .push(pick_list(themes, Some(theme), |selected| {
                            AppMsg::ChangeTheme(selected)
                        }))
                        .align_y(Vertical::Center)
                        .spacing(5),
                )
                .align_right(Length::Fill),
            )
            .align_y(Vertical::Center)
            .spacing(5)
            .wrap();
        // .clip(true);

        container(row)
            .align_y(Vertical::Bottom)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();

                container::Style {
                    background: Some(Background::Color(palette.background.weak.color)),
                    ..Default::default()
                }
            })
            .padding([0, 10])
            .into()
    }

    pub fn update(&mut self, message: FooterMsg) -> Task<AppMsg> {
        match message {
            FooterMsg::Error(s) => {
                self.status_line = s;
            }
            FooterMsg::LastUpdate(dt) => {
                self.status_line = format!("Updated: {}", dt.format("%d/%m/%Y %r"));
                self.last_update = Some(dt);
            }
            FooterMsg::Url(url) => {
                if self.status_line != url {
                    self.status_line = url;
                }
            }
            FooterMsg::NoUrl => match self.last_update.as_ref() {
                Some(dt) => self.status_line = format!("Updated: {}", dt.format("%d/%m/%Y %r")),
                None => self.status_line.clear(),
            },
            FooterMsg::Scale(scale) => {
                self.scale = scale;
            }
            FooterMsg::IndexStats { stats, category } => {
                self.current_index_stats = Some(stats);
                self.index_stats
                    .entry(category.as_str())
                    .and_modify(|s| *s = stats)
                    .or_insert(stats);
                return Task::done(AppMsg::SaveConfig);
            }
            FooterMsg::CurrentIndex(category) => {
                self.current_index_stats = self
                    .index_stats
                    .get(category.as_str())
                    .map(ToOwned::to_owned);
            }
        }
        Task::none()
    }
}
