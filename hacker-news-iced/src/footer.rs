use crate::app::AppMsg;
use chrono::{DateTime, FixedOffset, Local, Utc};
use chrono_tz::America::New_York;
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
    pub index_stats: Option<IndexStats>,
}

#[derive(Debug, Clone)]
pub enum FooterMsg {
    Error(String),
    LastUpdate(DateTime<Local>),
    Url(String),
    NoUrl,
    Scale(f64),
    IndexStats(IndexStats),
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
                    .width(Length::Fill.enclose(Length::Fill)),
            )
            .push(
                container(
                    Row::new()
                        .push_maybe(
                            self.index_stats
                                .as_ref()
                                .and_then(|stats| {
                                    DateTime::<Utc>::from_timestamp(stats.built_on as i64, 0)
                                })
                                .and_then(|dt| Some(dt.with_timezone(&New_York)))
                                .map(|dt| text(dt.format("%d/%m/%y %H:%M,").to_string())),
                        )
                        .push_maybe(self.index_stats.as_ref().and_then(|stats| {
                            (stats.build_time.as_secs() / 60 != 0)
                                .then(|| text(format!("{} min", stats.build_time.as_secs() / 60)))
                        }))
                        .push_maybe(self.index_stats.as_ref().map(|stats| {
                            text(format!("{} secs,", stats.build_time.as_secs() % 60))
                        }))
                        .push_maybe(self.index_stats.as_ref().map(|stats| {
                            text(format!(
                                "docs: {}, comments: {}",
                                stats.total_documents, stats.total_comments
                            ))
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
            .clip(true);

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
            FooterMsg::IndexStats(stats) => {
                self.index_stats = Some(stats);
                return Task::done(AppMsg::SaveConfig);
            }
        }
        Task::none()
    }
}
