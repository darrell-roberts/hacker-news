//! Header view.
use crate::{ArticleSelection, theme::Theme};
use gpui::{
    App, AppContext as _, BorrowAppContext, BoxShadow, Div, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, Stateful, StatefulInteractiveElement as _,
    Styled, Window, black, div, point, prelude::FluentBuilder, px, rems, rgb, white, yellow,
};
use hacker_news_api::ArticleType;

/// Header view
pub struct Header {
    counts: [(usize, SharedString); 5],
    categories: [(ArticleType, SharedString); 6],
}

impl Header {
    /// Create a new header view.
    ///
    ///
    /// # Arguments
    ///
    /// * `_cx` - A mutable reference to the current window context.
    /// * `app` - A mutable reference to the application instance.
    ///
    /// # Returns
    ///
    /// Returns an `Entity<Self>` representing the newly created header view.
    pub fn new(_window: &mut Window, app: &mut App) -> Entity<Self> {
        app.new(|_cx| Self {
            counts: [25, 50, 75, 100, 500].map(|n| (n, format!("{n}").into())),
            categories: [
                ArticleType::Top,
                ArticleType::Best,
                ArticleType::New,
                ArticleType::Ask,
                ArticleType::Show,
                ArticleType::Job,
            ]
            .map(|category| (category, category.as_str().into())),
        })
    }
}

/// Create a button with the given label.
fn mk_button(label: SharedString) -> Stateful<Div> {
    div()
        .bg(rgb(0x404040))
        .shadow(vec![BoxShadow {
            color: black().opacity(0.75),
            offset: point(px(2.0), px(2.0)),
            blur_radius: px(2.0),
            spread_radius: px(2.0),
        }])
        .id(label.clone())
        .child(label)
        .cursor_pointer()
        .rounded(rems(0.75))
        .hover(|style| style.opacity(1.0))
        .active(|style| style.shadow_none())
        .opacity(0.75)
        .p_1()
}

impl Render for Header {
    fn render(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme: Theme = window.appearance().into();

        let article_selection = cx.global::<ArticleSelection>();
        let mk_article_type = |(article_type, label): &(ArticleType, SharedString)| {
            let article_type = *article_type;
            let label = label.clone();
            mk_button(label)
                .when_else(
                    article_type == article_selection.viewing_article_type,
                    |div| div.bg(theme.button_active()).text_color(white()),
                    |div| div.bg(theme.button_inactive()).text_color(black()),
                )
                .on_click(move |_event, _window, app| {
                    app.update_global(|state: &mut ArticleSelection, _cx| {
                        state.viewing_article_type = article_type;
                    });
                })
        };

        let top_best_new = self.categories[0..3].iter().map(mk_article_type);
        let ask_show_job = self.categories[3..].iter().map(mk_article_type);
        let article_limits = self.counts.iter().cloned().map(|(article_count, label)| {
            mk_button(label)
                .when_else(
                    article_count == article_selection.viewing_article_total,
                    |div| div.bg(theme.button_active()).text_color(white()),
                    |div| div.bg(theme.button_inactive()).text_color(black()),
                )
                .on_click(move |_event, _window, app| {
                    app.update_global(|state: &mut ArticleSelection, _cx| {
                        state.viewing_article_total = article_count;
                    });
                })
        });

        div()
            .flex()
            .flex_row()
            .text_size(px(20.0))
            .text_color(yellow())
            .gap_x(px(10.0))
            .w_full()
            .justify_center()
            .m_1()
            .pb_1()
            .children(top_best_new)
            .child(div().border_4())
            .children(ask_show_job)
            .child(div().border_4())
            .children(article_limits)
            .px_1()
    }
}
