use crate::{
    event::{ClientEvent, Event, EventHandler},
    text::{parse_date, render_rich_text},
};
use egui::{
    os::OperatingSystem, style::Spacing, Color32, CursorIcon, Frame, Margin, RichText, Rounding,
    Style, TextStyle, Vec2,
};
use hacker_news_api::Item;
use log::error;
use tokio::sync::mpsc::UnboundedSender;

/// Application State.
pub struct HackerNewsApp {
    /// Top stories.
    top_stories: Vec<Item>,
    /// Active comments being viewed.
    comments: Vec<Item>,
    /// Event handler for background events.
    event_handler: EventHandler,
    /// Toggle comment view window.
    showing_comments: bool,
    /// API request in progress.
    fetching: bool,
    /// Trail of comments navigated.
    comment_trail: Vec<Vec<Item>>,
    /// Emit local events.
    local_sender: UnboundedSender<Event>,
    /// Active item when reading comments.
    active_item: Option<Item>,
    /// Parent comment trail.
    parent_comments: Vec<Item>,
    /// Number of articles to show.
    showing: usize,
}

impl HackerNewsApp {
    /// Create a new [`HackerNewsApp`].
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        event_handler: EventHandler,
        local_sender: UnboundedSender<Event>,
    ) -> Self {
        Self {
            event_handler,
            top_stories: Vec::new(),
            comments: Vec::new(),
            showing_comments: false,
            fetching: true,
            comment_trail: Vec::new(),
            local_sender,
            active_item: None,
            parent_comments: Vec::new(),
            showing: 50,
        }
    }

    /// Handle background emitted events.
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::TopStories(ts) => {
                self.showing = ts.len();
                self.top_stories = ts;
            }
            Event::Comments(comments, parent) => {
                if let Some(comment) = parent {
                    self.comment_trail.push(std::mem::take(&mut self.comments));
                    self.parent_comments.push(comment);
                } else {
                    self.comment_trail = Vec::new();
                    self.parent_comments = Vec::new();
                }
                self.comments = comments;
            }
            Event::Back => {
                match self.comment_trail.pop() {
                    Some(cs) => self.comments = cs,
                    None => self.comments = Vec::new(),
                };
                self.parent_comments.pop();
            }
        }
        self.fetching = false;
    }

    /// Handle background emitted events.
    fn handle_next_event(&mut self) {
        self.event_handler
            .next_event()
            .map(|event| self.handle_event(event))
            .unwrap_or_default();
    }

    /// Render the articles.
    fn render_articles(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (article, index) in self.top_stories.iter().zip(1..) {
                ui.horizontal(|ui| {
                    ui.label(format!("{index:>2}."));
                    if let Some(url) = article.url.as_deref() {
                        ui.style_mut().visuals.hyperlink_color = Color32::BLACK;
                        ui.hyperlink_to(
                            RichText::new(article.title.as_deref().unwrap_or("No title"))
                                .strong()
                                .color(Color32::BLACK),
                            url,
                        );
                    } else {
                        ui.label(article.title.as_deref().unwrap_or("No title"));
                    }
                    ui.style_mut().override_text_style = Some(TextStyle::Small);
                    ui.style_mut().spacing = Spacing {
                        item_spacing: Vec2 { y: 1., x: 2. },
                        ..Default::default()
                    };
                    ui.label(RichText::new(format!("{} points", article.score)).italics());
                    ui.label(RichText::new("by").italics());
                    ui.label(RichText::new(&article.by).italics());
                    if let Some(time) = parse_date(article.time) {
                        ui.label(RichText::new(time).italics());
                    }
                    if !article.kids.is_empty()
                        && ui.button(format!("{}", article.kids.len())).clicked()
                    {
                        self.showing_comments = true;
                        self.comments = Vec::new();
                        self.fetching = true;
                        self.active_item = Some(article.to_owned());
                        if let Err(err) = self
                            .event_handler
                            .emit(ClientEvent::Comments(article.kids.clone(), None))
                        {
                            error!("Failed to emit comments: {err}");
                        }
                    }
                });
            }
            ui.allocate_space(ui.available_size());
        });
    }

    /// Render comments if requested.
    fn render_comments(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if !self.comments.is_empty() && self.showing_comments {
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
                .open(&mut self.showing_comments)
                .show(ctx, |ui| {
                    if let Some(item) = self.active_item.as_ref() {
                        if !self.comment_trail.is_empty() && ui.button("back").clicked() {
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
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for parent in self.parent_comments.iter() {
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

                        for comment in self.comments.iter() {
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
                                    self.fetching = true;
                                    if let Err(err) =
                                        self.event_handler.emit(ClientEvent::Comments(
                                            comment.kids.clone(),
                                            Some(comment.to_owned()),
                                        ))
                                    {
                                        error!("Failed to emit comments: {err}");
                                    }
                                }
                            });
                            // ui.separator();
                        }
                    })
                });
        }
    }
}

impl eframe::App for HackerNewsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_next_event();

        if ctx.os() == OperatingSystem::Mac {
            ctx.set_pixels_per_point(2.5);
        } else {
            ctx.set_pixels_per_point(3.0);
        }

        if self.fetching {
            ctx.set_cursor_icon(CursorIcon::Progress);
        } else {
            ctx.set_cursor_icon(CursorIcon::Default);
        }

        let frame = Frame {
            // fill: Color32::LIGHT_BLUE,
            fill: Color32::from_rgb(189, 200, 204),
            inner_margin: Margin {
                left: 5.,
                right: 5.,
                top: 5.,
                bottom: 5.,
            },
            ..Default::default()
        };

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.visuals_mut().widgets.noninteractive.fg_stroke.color = Color32::BLACK;
            ui.visuals_mut().widgets.active.fg_stroke.color = Color32::BLACK;
            ui.visuals_mut().widgets.hovered.fg_stroke.color = Color32::BLACK;

            // Header
            ui.horizontal(|ui| {
                ui.label(format!("Total: {}", self.top_stories.len()));
                if ui.button("Reload").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(ClientEvent::TopStories(self.showing))
                        .unwrap_or_default();
                }
                if ui.selectable_label(self.showing == 25, "25").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(ClientEvent::TopStories(25))
                        .unwrap_or_default();
                }
                if ui.selectable_label(self.showing == 50, "50").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(ClientEvent::TopStories(50))
                        .unwrap_or_default();
                }
                if ui.selectable_label(self.showing == 75, "75").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(ClientEvent::TopStories(75))
                        .unwrap_or_default();
                }
                if ui.selectable_label(self.showing == 100, "100").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(ClientEvent::TopStories(100))
                        .unwrap_or_default();
                }
                if self.fetching {
                    ui.spinner();
                }
            });

            self.render_comments(ctx, ui);
            self.render_articles(ui);
        });
    }
}
