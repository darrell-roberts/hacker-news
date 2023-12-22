use crate::{
    app::scroll_delta,
    event::{ClientEvent, Event, EventHandler},
    text::{parse_date, render_rich_text},
};
use egui::{
    containers::Frame, epaint::Shadow, style::Spacing, widgets::Widget, Button, Color32, Id,
    Margin, RichText, Rounding, Stroke, Vec2,
};
use hacker_news_api::Item;
use log::error;
use tokio::sync::mpsc::UnboundedSender;

pub struct CommentItem {
    pub comments: Vec<Item>,
    pub parent: Option<Item>,
    pub id: Id,
}

/// Comment state data.
#[derive(Default)]
pub struct CommentsState {
    /// Active comments being viewed.
    pub comments: Vec<Item>,
    /// Trail of comments navigated.
    pub comment_trail: Vec<CommentItem>,
    /// Parent comment trail.
    pub parent_comments: Vec<Item>,
    /// Active item when reading comments.
    pub active_item: Option<Item>,
}

/// Renderer for comment window.
pub struct Comments<'a> {
    /// Emit local events.
    pub local_sender: &'a UnboundedSender<Event>,
    /// API request in progress.
    pub fetching: &'a mut bool,
    /// Event handler for background events.
    pub event_handler: &'a EventHandler,
    /// Comments state.
    pub comments_state: &'a CommentsState,
    /// Flags for open/closing comment windows.
    pub open_comments: &'a mut Vec<bool>,
}

impl<'a> Comments<'a> {
    /// Render comments if requested.
    pub fn render(&mut self, ctx: &egui::Context, _ui: &mut egui::Ui) {
        let frame = Frame::none()
            .fill(Color32::from_rgb(246, 247, 176))
            .inner_margin(Margin {
                left: 5.,
                right: 5.,
                top: 5.,
                bottom: 5.,
            })
            .rounding(Rounding {
                nw: 8.,
                ne: 8.,
                sw: 8.,
                se: 8.,
            })
            .stroke(Stroke {
                color: Color32::BLACK,
                width: 1.,
            })
            .shadow(Shadow::small_light());

        for (comment_item, index) in self.comments_state.comment_trail.iter().zip(0..) {
            let open = &mut self.open_comments[index];
            egui::Window::new("")
                .id(comment_item.id)
                .frame(frame)
                .vscroll(true)
                .open(open)
                .collapsible(false)
                .show(ctx, |ui| {
                    let render_by = |ui: &mut egui::Ui, item: &Item, comments: bool| {
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
                                self.local_sender
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
                    };

                    let scroll_delta = scroll_delta(ui);
                    ui.scroll_with_delta(scroll_delta);
                    if let Some(item) = self.comments_state.active_item.as_ref() {
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
                        render_by(ui, item, true);
                    }
                    if let Some(parent_comment) = comment_item.parent.as_ref() {
                        ui.style_mut().visuals.override_text_color = Some(Color32::DARK_GRAY);
                        render_rich_text(parent_comment.text.as_deref().unwrap_or_default(), ui);
                        render_by(ui, parent_comment, true);
                    }
                    ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);

                    for comment in comment_item.comments.iter() {
                        Frame::none()
                            .fill(Color32::LIGHT_YELLOW)
                            .outer_margin(Margin {
                                top: 5.,
                                left: 10.,
                                right: 10.,
                                bottom: 5.,
                            })
                            .inner_margin(Margin {
                                top: 10.,
                                left: 10.,
                                right: 10.,
                                bottom: 10.,
                            })
                            .rounding(Rounding {
                                nw: 8.,
                                ne: 8.,
                                sw: 8.,
                                se: 8.,
                            })
                            .show(ui, |ui| {
                                render_rich_text(comment.text.as_deref().unwrap_or_default(), ui);

                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    ui.style_mut().spacing = Spacing {
                                        item_spacing: Vec2 { y: 1., x: 2. },
                                        ..Default::default()
                                    };
                                    ui.style_mut().visuals.override_text_color =
                                        Some(Color32::GRAY);
                                    render_by(ui, comment, false);
                                    if !comment.kids.is_empty() {
                                        ui.style_mut().visuals.override_text_color =
                                            Some(Color32::BLACK);
                                        let button =
                                            Button::new(format!("💬{}", comment.kids.len()))
                                                .fill(Color32::LIGHT_YELLOW)
                                                .ui(ui);

                                        if button.clicked() {
                                            *self.fetching = true;
                                            if let Err(err) =
                                                self.event_handler.emit(ClientEvent::Comments {
                                                    ids: comment.kids.clone(),
                                                    parent: Some(comment.to_owned()),
                                                    id: Id::new(comment.id),
                                                })
                                            {
                                                error!("Failed to emit comments: {err}");
                                            }
                                        }
                                    }
                                });
                                ui.set_width(ui.available_width());
                            });
                    }
                });
        }
    }
}
