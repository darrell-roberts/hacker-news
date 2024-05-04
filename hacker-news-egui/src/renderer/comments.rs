//! Render the comment windows.
use super::{
    styles::{
        comment_bubble_color, comment_bubble_frame, comment_bubble_text, comment_window_frame,
    },
    text::{parse_date, render_rich_text},
};
use crate::{
    app::{HackerNewsApp, MutableWidgetState},
    event::Event,
    renderer::scroll_delta,
};
use egui::{style::Spacing, widgets::Widget, Align, Button, Color32, Id, Layout, RichText, Vec2};
use hacker_news_api::Item;

/// Render comments if requested.
pub fn render(app_state: &HackerNewsApp, mutable_state: &MutableWidgetState, ui: &mut egui::Ui) {
    if let Some((index, _)) = mutable_state
        .viewing_comments
        .iter()
        .enumerate()
        .rev()
        .find(|&(_, open)| *open)
    {
        let comment_item = &app_state.comments_state.comment_trail[index];

        egui::ScrollArea::vertical()
            .id_source(Id::new(comment_item.id))
            .show(ui, |ui| {
                comment_window_frame(&app_state.theme).show(ui, |ui| {
                    let scroll_delta = scroll_delta(ui);
                    ui.scroll_with_delta(scroll_delta);

                    let ids = app_state
                        .comments_state
                        .comment_trail
                        .iter()
                        .filter(|item| item.open)
                        .filter_map(|item| item.parent.as_ref())
                        .map(|item| format!("{}", item.id))
                        .collect::<Vec<_>>();

                    let trail = ids.as_slice().join(" > ");

                    // breadcrumb and close icon.
                    ui.horizontal(|ui| {
                        if !trail.is_empty() {
                            ui.label(trail);
                        }

                        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                            ui.add_space(4.);
                            if ui.button("X").clicked() {
                                app_state.emit(Event::CloseComment(index));
                            }
                        });
                    });

                    if let Some(item) = app_state.comments_state.active_item.as_ref() {
                        // ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);
                        // ui.style_mut().visuals.hyperlink_color = Color32::BLACK;
                        ui.style_mut().visuals.override_text_color =
                            Some(comment_bubble_text(&app_state.theme));
                        if let Some(title) = item.title.as_deref() {
                            match item.url.as_deref() {
                                Some(url) => ui
                                    .hyperlink_to(RichText::new(title).heading(), url)
                                    .on_hover_text(url),
                                None => ui.heading(title),
                            };
                        }
                        if let Some(text) = item.text.as_deref() {
                            render_rich_text(text, ui);
                        }
                        render_by(ui, app_state, item, true);
                    }
                    if let Some(parent_comment) = comment_item.parent.as_ref() {
                        // ui.style_mut().visuals.override_text_color = Some(match app_state.theme {
                        //     eframe::Theme::Dark => Color32::BLACK,
                        //     eframe::Theme::Light => Color32::DARK_GRAY,
                        // });
                        render_rich_text(parent_comment.text.as_deref().unwrap_or_default(), ui);
                        render_by(ui, app_state, parent_comment, true);
                    }
                    ui.style_mut().visuals.override_text_color =
                        Some(comment_bubble_text(&app_state.theme));

                    render_comments(comment_item, app_state, ui);
                });
            });
    }
}

fn render_by(ui: &mut egui::Ui, app_state: &HackerNewsApp, item: &Item, comments: bool) {
    ui.horizontal(|ui| {
        ui.style_mut().spacing = Spacing {
            item_spacing: Vec2 { y: 1., x: 2. },
            ..Default::default()
        };

        ui.style_mut().visuals.hyperlink_color = Color32::GRAY;
        ui.style_mut().visuals.override_text_color = Some(Color32::GRAY);
        if ui
            .link(RichText::new(&item.by).italics().color(Color32::GRAY))
            .clicked()
        {
            app_state.emit(Event::FetchUser(item.by.clone()));
        };

        if let Some(time) = parse_date(item.time) {
            ui.label(RichText::new(time).italics());
        }
        ui.add_space(5.0);
        if comments {
            ui.label(format!("💬{}", item.kids.len()));
        }
    });
}

fn render_comments(
    comment_item: &crate::app::CommentItem,
    app_state: &HackerNewsApp,
    ui: &mut egui::Ui,
) {
    let search_filter = |comment: &&Item| {
        app_state.search.is_empty()
            || comment
                .text
                .as_deref()
                .map(|text| {
                    text.split_whitespace().any(|word| {
                        app_state.search.split_whitespace().any(|search_term| {
                            word.to_lowercase().contains(&search_term.to_lowercase())
                        })
                    })
                })
                .unwrap_or(false)
    };

    for comment in comment_item.comments.iter().filter(search_filter) {
        comment_bubble_frame(&app_state.theme).show(ui, |ui| {
            render_rich_text(comment.text.as_deref().unwrap_or_default(), ui);

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.style_mut().spacing = Spacing {
                    item_spacing: Vec2 { y: 1., x: 2. },
                    ..Default::default()
                };
                ui.style_mut().visuals.override_text_color = Some(Color32::GRAY);
                render_by(ui, app_state, comment, false);
                if !comment.kids.is_empty() {
                    ui.style_mut().visuals.override_text_color =
                        Some(comment_bubble_text(&app_state.theme));
                    let button = Button::new(format!("💬{}", comment.kids.len()))
                        .fill(comment_bubble_color(&app_state.theme))
                        .ui(ui);

                    if button.clicked() {
                        app_state.emit(Event::FetchComments {
                            ids: comment.kids.clone(),
                            parent: Some(comment.to_owned()),
                            id: Id::new(comment.id),
                            active_item: None,
                        });
                    }
                }
            });
            ui.set_width(ui.available_width());
        });
    }
}
