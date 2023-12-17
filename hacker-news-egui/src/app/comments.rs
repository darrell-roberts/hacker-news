use crate::{
    app::scroll_delta,
    event::{ClientEvent, Event, EventHandler},
    text::{parse_date, render_rich_text},
};
use egui::{style::Spacing, Color32, Frame, Margin, RichText, Rounding, Style, TextStyle, Vec2};
use hacker_news_api::Item;
use log::error;
use tokio::sync::mpsc::UnboundedSender;

/// Comment state data.
#[derive(Default)]
pub struct CommentsState {
    /// Active comments being viewed.
    pub comments: Vec<Item>,
    /// Trail of comments navigated.
    pub comment_trail: Vec<Vec<Item>>,
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
    /// Toggle comment view window.
    pub showing_comments: &'a mut bool,
    /// Comments state.
    pub comments_state: &'a CommentsState,
}

impl<'a> Comments<'a> {
    /// Render comments if requested.
    pub fn render(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let frame = Frame {
            fill: Color32::LIGHT_YELLOW,
            inner_margin: Margin {
                left: 5.,
                right: 5.,
                top: 5.,
                bottom: 5.,
            },
            rounding: Rounding {
                nw: 8.,
                ne: 8.,
                sw: 8.,
                se: 8.,
            },
            ..Default::default()
        };

        let width = ui.available_width() - (ui.available_width() * 0.05);
        let height = ui.available_height() - (ui.available_height() * 0.15);

        egui::Window::new("")
            .frame(frame)
            .default_width(width)
            .default_height(height)
            .open(self.showing_comments)
            .show(ctx, |ui| {
                if let Some(item) = self.comments_state.active_item.as_ref() {
                    if !self.comments_state.comment_trail.is_empty() && ui.button("back").clicked()
                    {
                        if let Err(err) = self.local_sender.send(Event::Back) {
                            error!("Failed to send Back: {err}");
                        }
                        ctx.request_repaint();
                    }
                    ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);
                    if let Some(title) = item.title.as_deref() {
                        ui.heading(title);
                        ui.horizontal(|ui| {
                            ui.set_style(Style {
                                override_text_style: Some(TextStyle::Small),
                                ..Default::default()
                            });
                            ui.style_mut().spacing = Spacing {
                                item_spacing: Vec2 { y: 1., x: 2. },
                                ..Default::default()
                            };

                            ui.label(RichText::new("by").italics());
                            ui.label(RichText::new(&item.by).italics());
                            if let Some(time) = parse_date(item.time) {
                                ui.label(RichText::new(time).italics());
                            }
                            ui.label(format!("[{}]", item.kids.len()));
                        });
                    }
                    if let Some(text) = item.text.as_deref() {
                        render_rich_text(text, ui);
                    }
                    // ui.separator();
                }
                let scroll_delta = scroll_delta(ui);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.scroll_with_delta(scroll_delta);
                    for parent in self.comments_state.parent_comments.iter() {
                        ui.style_mut().visuals.override_text_color = Some(Color32::GRAY);
                        render_rich_text(parent.text.as_deref().unwrap_or_default(), ui);
                        ui.horizontal(|ui| {
                            ui.set_style(Style {
                                override_text_style: Some(TextStyle::Small),
                                ..Default::default()
                            });
                            ui.style_mut().spacing = Spacing {
                                item_spacing: Vec2 { y: 1., x: 2. },
                                ..Default::default()
                            };
                            ui.label("by");
                            ui.label(&parent.by);
                            if let Some(time) = parse_date(parent.time) {
                                ui.label(RichText::new(time).italics());
                            }
                            ui.label(format!("[{}]", parent.kids.len()));
                        });
                        ui.style_mut().visuals.override_text_color = None;
                    }

                    ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);

                    for comment in self.comments_state.comments.iter() {
                        render_rich_text(comment.text.as_deref().unwrap_or_default(), ui);

                        ui.horizontal(|ui| {
                            ui.set_style(Style {
                                override_text_style: Some(TextStyle::Small),
                                ..Default::default()
                            });
                            ui.style_mut().spacing = Spacing {
                                item_spacing: Vec2 { y: 1., x: 2. },
                                ..Default::default()
                            };
                            // if ui.button(format!("id: {}", comment.id)).clicked() {
                            //     ui.output_mut(|p| p.copied_text = format!("{}", comment.id));
                            // }
                            ui.label(RichText::new("by").italics());
                            ui.label(RichText::new(&comment.by).italics());
                            if let Some(time) = parse_date(comment.time) {
                                ui.label(RichText::new(time).italics());
                            }
                            if !comment.kids.is_empty()
                                && ui.button(format!("{}", comment.kids.len())).clicked()
                            {
                                *self.fetching = true;
                                if let Err(err) = self.event_handler.emit(ClientEvent::Comments(
                                    comment.kids.clone(),
                                    Some(comment.to_owned()),
                                )) {
                                    error!("Failed to emit comments: {err}");
                                }
                            }
                        });
                    }
                })
            });
    }
}
