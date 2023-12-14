use egui::{Color32, RichText, Style, TextStyle};
use event::{ClientEvent, Event, EventHandler, SHUT_DOWN};
use hacker_news_api::Item;
use log::error;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;

pub enum ApiEvent {
    TopStories(Vec<Item>),
}

pub mod event;

#[tokio::main]
async fn main() {
    env_logger::init();

    let client = hacker_news_api::ApiClient::new().expect("Could not get api client");

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Hacker News",
        native_options,
        Box::new(|cc| {
            let frame = cc.egui_ctx.clone();
            let (sender, mut receiver) = mpsc::unbounded_channel::<ClientEvent>();
            let (local_sender, client_receiver) = mpsc::unbounded_channel::<Event>();

            sender
                .send(ClientEvent::TopStories)
                .expect("Failed to request initial top stories");

            let event_handler = EventHandler::new(sender, client_receiver);

            let _handle = tokio::spawn(async move {
                while !(SHUT_DOWN.load(Ordering::Acquire)) {
                    if let Some(data) = receiver.recv().await {
                        match data {
                            ClientEvent::TopStories => {
                                let top_stories = client.top_stories(50).await;
                                match top_stories {
                                    Ok(ts) => {
                                        if let Err(err) = local_sender.send(Event::TopStories(ts)) {
                                            error!("Failed to send top stories: {err}");
                                        }
                                        frame.request_repaint();
                                    }
                                    Err(err) => {
                                        error!("Failed to get top stories: {err}");
                                    }
                                }
                            }
                            ClientEvent::Comments(ids) => match client.items(&ids).await {
                                Ok(comments) => {
                                    match local_sender.send(Event::Comments(comments)) {
                                        Ok(_) => frame.request_repaint(),
                                        Err(err) => error!("Failed to send comments: {err}"),
                                    }
                                }
                                Err(err) => error!("Failed to get comments: {err}"),
                            },
                        }
                    }
                }
            });

            Box::new(HackerNewsApp::new(cc, event_handler))
        }),
    )
    .unwrap();
}

struct HackerNewsApp {
    top_stories: Vec<Item>,
    comments: Vec<Item>,
    event_handler: EventHandler,
    showing_comments: bool,
}

impl HackerNewsApp {
    fn new(_cc: &eframe::CreationContext<'_>, event_handler: EventHandler) -> Self {
        Self {
            event_handler,
            top_stories: Vec::new(),
            comments: Vec::new(),
            showing_comments: false,
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
                        ui.label(format!("{index}. "));
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
                            self.event_handler
                                .emit(ClientEvent::Comments(article.kids.clone()))
                                .unwrap();
                        }
                    });
                }
                ui.allocate_space(ui.available_size());
            });
        });
    }
}
