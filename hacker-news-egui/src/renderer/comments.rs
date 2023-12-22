use crate::{
    app::{HackerNewsApp, MutableWidgetState},
    event::Event,
    renderer::scroll_delta,
    text::{parse_date, render_rich_text},
};
use egui::{
    containers::Frame, epaint::Shadow, style::Spacing, widgets::Widget, Button, Color32, Id,
    Margin, RichText, Rounding, Stroke, Vec2,
};
use hacker_news_api::Item;

/// Renderer for comment window.
pub struct Comments<'a, 'b> {
    pub context: &'a egui::Context,
    pub app_state: &'a HackerNewsApp,
    pub mutable_state: &'b mut MutableWidgetState,
}

impl<'a, 'b> Comments<'a, 'b> {
    pub fn new(
        context: &'a egui::Context,
        app_state: &'a HackerNewsApp,
        mutable_state: &'b mut MutableWidgetState,
    ) -> Self {
        Self {
            context,
            app_state,
            mutable_state,
        }
    }

    /// Render comments if requested.
    pub fn render(&mut self, ctx: &egui::Context) {
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

        for (comment_item, index) in self.app_state.comments_state.comment_trail.iter().zip(0..) {
            egui::Window::new("")
                .id(comment_item.id)
                .frame(frame)
                .vscroll(true)
                .open(&mut self.mutable_state.viewing_comments[index])
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
                                self.app_state
                                    .local_sender
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
                    if let Some(item) = self.app_state.comments_state.active_item.as_ref() {
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
                                            self.app_state
                                                .local_sender
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
                });
        }
    }
}
