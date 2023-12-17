use crate::{
    event::{ClientEvent, Event, EventHandler},
    text::parse_date,
};
use comments::{Comments, CommentsState};
use egui::{
    os::OperatingSystem, style::Spacing, Color32, CursorIcon, Frame, Key, Margin, RichText,
    TextStyle, Vec2,
};
use hacker_news_api::Item;
use log::error;
use tokio::sync::mpsc::UnboundedSender;

mod comments;

/// Application State.
pub struct HackerNewsApp {
    /// Top stories.
    top_stories: Vec<Item>,
    /// Event handler for background events.
    event_handler: EventHandler,
    /// Toggle comment view window.
    showing_comments: bool,
    /// API request in progress.
    fetching: bool,
    /// Emit local events.
    local_sender: UnboundedSender<Event>,
    /// Number of articles to show.
    showing: usize,
    /// Articles visited.
    visited: Vec<usize>,
    /// Comments state.
    comments_state: CommentsState,
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
            fetching: true,
            local_sender,
            showing: 50,
            visited: Vec::new(),
            comments_state: Default::default(),
            showing_comments: false,
        }
    }

    /// Handle background emitted events.
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::TopStories(ts) => {
                self.showing = ts.len();
                self.top_stories = ts;
                self.visited = Vec::new();
            }
            Event::Comments(comments, parent) => {
                if let Some(comment) = parent {
                    self.comments_state
                        .comment_trail
                        .push(std::mem::take(&mut self.comments_state.comments));
                    self.comments_state.parent_comments.push(comment);
                } else {
                    self.comments_state.comment_trail = Vec::new();
                    self.comments_state.parent_comments = Vec::new();
                }
                self.comments_state.comments = comments;
            }
            Event::Back => {
                match self.comments_state.comment_trail.pop() {
                    Some(cs) => self.comments_state.comments = cs,
                    None => self.comments_state.comments = Vec::new(),
                };
                self.comments_state.parent_comments.pop();
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
        let scroll_delta = scroll_delta(ui);
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.scroll_with_delta(scroll_delta);
            for (article, index) in self.top_stories.iter().zip(1..) {
                ui.horizontal(|ui| {
                    ui.label(format!("{index:>2}."));
                    if let Some(url) = article.url.as_deref() {
                        ui.style_mut().visuals.hyperlink_color = if self.visited.contains(&index) {
                            Color32::DARK_GRAY
                        } else {
                            Color32::BLACK
                        };
                        if ui
                            .hyperlink_to(
                                RichText::new(article.title.as_deref().unwrap_or("No title"))
                                    .strong()
                                    .color(Color32::BLACK),
                                url,
                            )
                            .clicked()
                        {
                            self.visited.push(index);
                        }
                    } else if self.visited.contains(&index) {
                        ui.label(
                            RichText::new(article.title.as_deref().unwrap_or("No title"))
                                .color(Color32::DARK_GRAY),
                        );
                    } else {
                        ui.label(article.title.as_deref().unwrap_or("No title"));
                    }
                    if self.visited.contains(&index) {
                        ui.style_mut().visuals.override_text_color = Some(Color32::DARK_GRAY);
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
                        self.comments_state.comments = Vec::new();
                        self.fetching = true;
                        self.comments_state.active_item = Some(article.to_owned());
                        self.visited.push(index);
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
        if !self.comments_state.comments.is_empty() && self.showing_comments {
            Comments {
                local_sender: &self.local_sender,
                fetching: &mut self.fetching,
                event_handler: &self.event_handler,
                showing_comments: &mut self.showing_comments,
                comments_state: &self.comments_state,
            }
            .render_comments(ctx, ui);
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
            // fill: Color32::from_rgb(189, 200, 204),
            fill: Color32::from_rgb(245, 243, 240),
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

            ui.add_space(2.);
            self.render_comments(ctx, ui);
            self.render_articles(ui);
        });
    }
}

fn scroll_delta(ui: &mut egui::Ui) -> Vec2 {
    let mut scroll_delta = Vec2::ZERO;
    ui.input_mut(|input| {
        if input.key_released(Key::PageDown) {
            scroll_delta.y -= ui.available_height();
        }
        if input.key_released(Key::PageUp) {
            scroll_delta.y += ui.available_height();
        }
        if input.key_released(Key::ArrowDown) {
            scroll_delta.y -= 24.0;
        }
        if input.key_released(Key::ArrowUp) {
            scroll_delta.y += 24.0;
        }
        if input.key_released(Key::Home) {
            scroll_delta.y = f32::MAX;
        }
        if input.key_released(Key::End) {
            scroll_delta.y = f32::MIN;
        }
    });
    scroll_delta
}
