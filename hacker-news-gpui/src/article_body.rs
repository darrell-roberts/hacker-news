//! View for the article body text
use crate::{
    rich_text::{ViewStyledText, rich_text_runs, url_ranges},
    theme::Theme,
};
use gpui::{
    App, AppContext as _, Entity, InteractiveText, Render, SharedString, StyledText, prelude::*,
    px, rems,
};
use std::rc::Rc;

/// View for article body.
pub struct ArticleBodyView {
    /// Article body.
    pub article_text: Rc<ViewStyledText>,
    /// Author.
    pub author: SharedString,
    /// Age.
    pub age: SharedString,
}

impl ArticleBodyView {
    pub fn new(
        app: &mut App,
        article_text: Rc<ViewStyledText>,
        author: SharedString,
        age: SharedString,
    ) -> Entity<Self> {
        app.new(|_cx| Self {
            article_text,
            author,
            age,
        })
    }
}

impl Render for ArticleBodyView {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme: Theme = window.appearance().into();
        let view_styled_text = self.article_text.clone();

        gpui::div()
            .mb_2()
            .bg(theme.article_text())
            .border_1()
            .border_color(theme.border())
            .shadow_md()
            .child(
                gpui::div().p_1().child(
                    InteractiveText::new(
                        "article_body",
                        StyledText::new(view_styled_text.text.clone())
                            .with_runs(rich_text_runs(theme, &view_styled_text.layout).collect()),
                    )
                    .on_click(
                        url_ranges(&view_styled_text.layout),
                        move |index, _window, app| {
                            if let Some(url) = view_styled_text.urls.get(index) {
                                app.open_url(url);
                            }
                        },
                    ),
                ),
            )
            .child(
                gpui::div()
                    .border_t_1()
                    .border_color(theme.border())
                    .p_1()
                    .flex()
                    .flex_row()
                    .italic()
                    .text_size(rems(0.75))
                    .gap_x(px(5.0))
                    .child(self.author.clone())
                    .child(self.age.clone()),
            )
    }
}
