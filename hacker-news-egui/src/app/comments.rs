use crate::{
    app::scroll_delta,
    event::{ClientEvent, Event, EventHandler},
    text::{parse_date, render_rich_text},
};
use egui::{
    style::Spacing, widgets::Widget, Button, Color32, Frame, Id, Margin, RichText, Rounding,
    Separator, Style, TextStyle, Vec2,
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

        for (comment_item, index) in self.comments_state.comment_trail.iter().zip(0..) {
            let open = &mut self.open_comments[index];
            egui::Window::new("")
                .id(comment_item.id)
                .frame(frame)
                .vscroll(true)
                .open(open)
                .collapsible(false)
                .show(ctx, |ui| {
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
                            render_by(ui, item);
                        }
                        if let Some(text) = item.text.as_deref() {
                            render_rich_text(text, ui);
                        }
                    }
                    if let Some(parent_comment) = comment_item.parent.as_ref() {
                        ui.style_mut().visuals.override_text_color = Some(Color32::DARK_GRAY);
                        render_rich_text(parent_comment.text.as_deref().unwrap_or_default(), ui);
                        render_by(ui, parent_comment);
                        ui.add_space(5.);
                        ui.separator();
                    }
                    ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);

                    for comment in comment_item.comments.iter() {
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
                            ui.add_space(5.);

                            if !comment.kids.is_empty() {
                                ui.style_mut().visuals.override_text_color = Some(Color32::BLACK);
                                let button = Button::new(format!("💬{}", comment.kids.len()))
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

                        ui.add(Separator::default().spacing(25.0));
                    }
                });
        }
    }
}

fn render_by(ui: &mut egui::Ui, item: &Item) {
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
        ui.add_space(5.0);
        ui.label(format!("[{}]", item.kids.len()));
    });
}
