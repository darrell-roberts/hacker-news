use crate::app::{App, AppMsg};
use iced::{
    alignment::Vertical,
    font::{Style, Weight},
    widget::{container, pick_list, text, Row},
    Background, Element, Font, Length, Theme,
};

impl App {
    pub fn render_footer(&self) -> Element<'_, AppMsg> {
        let themes = Theme::ALL;

        let light_font = || Font {
            style: Style::Italic,
            weight: Weight::Light,
            ..Default::default()
        };

        let row = Row::new()
            .push(text(&self.status_line).font(light_font()))
            .push(
                container(
                    Row::new()
                        .push(text(format!("Scale: {:.2}", self.scale)).font(light_font()))
                        .push(pick_list(themes, Some(&self.theme), |selected| {
                            AppMsg::ChangeTheme(selected)
                        }))
                        .align_y(Vertical::Center)
                        .spacing(5),
                )
                .align_right(Length::Fill),
            )
            .align_y(Vertical::Center)
            .spacing(25);

        container(row)
            .align_y(Vertical::Bottom)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();

                container::Style {
                    background: Some(Background::Color(palette.background.strong.color)),
                    ..Default::default()
                }
            })
            .padding([0, 10])
            .into()
    }
}
