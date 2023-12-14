use crate::event::{ClientEvent, Event, EventHandler};
use egui::{Color32, RichText, Style, TextStyle};
use hacker_news_api::Item;
use log::error;

pub struct HackerNewsApp {
    top_stories: Vec<Item>,
    comments: Vec<Item>,
    event_handler: EventHandler,
    showing_comments: bool,
    fetching: bool,
}

impl HackerNewsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, event_handler: EventHandler) -> Self {
        Self {
            event_handler,
            top_stories: Vec::new(),
            comments: Vec::new(),
            showing_comments: false,
            fetching: true,
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::TopStories(ts) => {
                self.top_stories = ts;
            }
            Event::Comments(comments) => {
                self.comments = comments;
            }
        }
        self.fetching = false;
    }

    fn handle_next_event(&mut self) {
        self.event_handler
            .next_event()
            .map(|event| self.handle_event(event))
            .unwrap_or_default();
    }
}

impl eframe::App for HackerNewsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_next_event();

        ctx.set_pixels_per_point(2.5);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.visuals_mut().widgets.noninteractive.fg_stroke.color = Color32::BLACK;
            ui.visuals_mut().widgets.active.fg_stroke.color = Color32::BLACK;
            ui.visuals_mut().widgets.hovered.fg_stroke.color = Color32::BLACK;

            ui.horizontal(|ui| {
                ui.label(format!("Total: {}", self.top_stories.len()));
                if ui.button("Reload").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(ClientEvent::TopStories)
                        .unwrap_or_default();
                }
                ui.horizontal(|ui| {
                    // ui.set_height(20.);
                    if self.fetching {
                        ui.centered_and_justified(|ui| {
                            ui.spinner();
                        });
                    }
                });
            });

            if !self.comments.is_empty() && self.showing_comments {
                egui::Window::new("Comments")
                    .open(&mut self.showing_comments)
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for comment in self.comments.iter() {
                                ui.label(
                                    comment
                                        .text
                                        .as_ref()
                                        .cloned()
                                        .map(http_sanitizer::sanitize_html)
                                        .unwrap_or_default(),
                                );
                                ui.horizontal(|ui| {
                                    ui.set_style(Style {
                                        override_text_style: Some(TextStyle::Small),
                                        ..Default::default()
                                    });
                                    ui.label("by");
                                    ui.label(&comment.by);
                                    ui.label(format!("{}", comment.kids.len()));
                                });
                                ui.separator();
                            }
                        })
                    });
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (article, index) in self.top_stories.iter().zip(1..) {
                    ui.horizontal(|ui| {
                        ui.label(format!("{index:>2}."));
                        ui.hyperlink_to(
                            RichText::new(article.title.as_deref().unwrap_or("nothing"))
                                .strong()
                                .color(Color32::BLACK),
                            article.url.as_deref().unwrap_or_default(),
                        );
                        ui.label("by");
                        ui.label(&article.by);
                        if ui.button(format!("[{}]", article.kids.len())).clicked() {
                            self.showing_comments = true;
                            self.comments = Vec::new();
                            self.fetching = true;
                            if let Err(err) = self
                                .event_handler
                                .emit(ClientEvent::Comments(article.kids.clone()))
                            {
                                error!("Failed to emit comments: {err}");
                            }
                        }
                    });
                }
                ui.allocate_space(ui.available_size());
            });
        });
    }
}
