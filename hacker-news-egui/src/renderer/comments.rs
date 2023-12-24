//! Render the comment windows.
use super::styles::{comment_bubble_frame, comment_window_frame};
use crate::{
    app::{HackerNewsApp, MutableWidgetState},
    event::Event,
    renderer::scroll_delta,
    text::{parse_date, render_rich_text},
};
use egui::{style::Spacing, widgets::Widget, Button, Color32, Id, RichText, Vec2};
use hacker_news_api::Item;
use tokio::sync::mpsc::UnboundedSender;

/// Render comments if requested.
pub fn render<'a>(
    context: &'a egui::Context,
    app_state: &'a HackerNewsApp,
    mutable_state: &mut MutableWidgetState,
) {
    for (comment_item, index) in app_state.comments_state.comment_trail.iter().zip(0..) {
        egui::Window::new("")
            .id(comment_item.id)
            .frame(comment_window_frame())
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
                    render_by(ui, &app_state.local_sender, item, true);
                }
                if let Some(parent_comment) = comment_item.parent.as_ref() {
                    ui.style_mut().visuals.override_text_color = Some(Color32::DARK_GRAY);
                    render_rich_text(parent_comment.text.as_deref().unwrap_or_default(), ui);
                    render_by(ui, &app_state.local_sender, parent_comment, true);
                }
                ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);

                render_comments(comment_item, &app_state.local_sender, ui);
            });
    }
}

fn render_by(ui: &mut egui::Ui, sender: &UnboundedSender<Event>, item: &Item, comments: bool) {
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
            sender
                .send(Event::FetchUser(item.by.clone()))
                .unwrap_or_default();
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
    sender: &UnboundedSender<Event>,
    ui: &mut egui::Ui,
) {
    for comment in comment_item.comments.iter() {
        comment_bubble_frame().show(ui, |ui| {
            render_rich_text(comment.text.as_deref().unwrap_or_default(), ui);

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.style_mut().spacing = Spacing {
                    item_spacing: Vec2 { y: 1., x: 2. },
                    ..Default::default()
                };
                ui.style_mut().visuals.override_text_color = Some(Color32::GRAY);
                render_by(ui, sender, comment, false);
                if !comment.kids.is_empty() {
                    ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);
                    let button = Button::new(format!("💬{}", comment.kids.len()))
                        .fill(Color32::LIGHT_YELLOW)
                        .ui(ui);

                    if button.clicked() {
                        sender
                            .send(Event::FetchComments {
                                ids: comment.kids.clone(),
                                parent: Some(comment.to_owned()),
                                id: Id::new(comment.id),
                                active_item: None,
                            })
                            .unwrap_or_default();
                    }
                }
            });
            ui.set_width(ui.available_width());
        });
    }
}
