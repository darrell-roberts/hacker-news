//! Render the comment windows.
use super::{
    styles::{comment_bubble_frame, comment_window_frame},
    text::{parse_date, render_rich_text},
};
use crate::{
    app::{HackerNewsApp, MutableWidgetState},
    event::Event,
    renderer::scroll_delta,
};
use egui::{style::Spacing, widgets::Widget, Button, Color32, Id, RichText, Vec2};
use hacker_news_api::Item;

/// Render comments if requested.
pub fn render<'a>(
    context: &'a egui::Context,
    app_state: &'a HackerNewsApp,
    mutable_state: &mut MutableWidgetState,
) {
    for (comment_item, index) in app_state.comments_state.comment_trail.iter().zip(0..) {
        egui::Window::new("")
            .id(comment_item.id)
            .frame(comment_window_frame(&app_state.theme))
            .vscroll(true)
            .open(&mut mutable_state.viewing_comments[index])
            .collapsible(false)
            .show(context, |ui| {
                let scroll_delta = scroll_delta(ui);
                ui.scroll_with_delta(scroll_delta);
                if let Some(item) = app_state.comments_state.active_item.as_ref() {
                    ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);
                    ui.style_mut().visuals.hyperlink_color = Color32::BLACK;
                    if let Some(title) = item.title.as_deref() {
                        match item.url.as_deref() {
                            Some(url) => ui.hyperlink_to(RichText::new(title).heading(), url),
                            None => ui.heading(title),
                        };
                    }
                    if let Some(text) = item.text.as_deref() {
                        render_rich_text(text, ui);
                    }
                    render_by(ui, app_state, item, true);
                }
                if let Some(parent_comment) = comment_item.parent.as_ref() {
                    ui.style_mut().visuals.override_text_color = Some(Color32::DARK_GRAY);
                    render_rich_text(parent_comment.text.as_deref().unwrap_or_default(), ui);
                    render_by(ui, app_state, parent_comment, true);
                }
                ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);

                render_comments(comment_item, app_state, ui);
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
    for comment in comment_item.comments.iter() {
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
                    ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);
                    let button = Button::new(format!("💬{}", comment.kids.len()))
                        .fill(Color32::LIGHT_YELLOW)
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
