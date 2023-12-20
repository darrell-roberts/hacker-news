use self::comments::CommentItem;
use crate::{
    event::{ClientEvent, Event, EventHandler},
    text::parse_date,
    SHUT_DOWN,
};
use comments::{Comments, CommentsState};
use egui::{
    os::OperatingSystem, style::Spacing, Color32, CursorIcon, Frame, Grid, Id, Key, Margin,
    RichText, TextStyle, Vec2,
};
use hacker_news_api::Item;
use log::error;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc::UnboundedSender;

mod comments;

#[derive(Eq, PartialEq)]
pub enum ArticleType {
    New,
    Best,
    Top,
}

/// Application State.
pub struct HackerNewsApp {
    /// Top stories.
    articles: Vec<Item>,
    /// Event handler for background events.
    event_handler: EventHandler,
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
    /// Errors.
    error: Option<String>,
    /// Viewing article type.
    article_type: ArticleType,
    /// Comment window open states.
    open_comments: Vec<bool>,
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
            articles: Vec::new(),
            fetching: true,
            local_sender,
            showing: 50,
            visited: Vec::new(),
            comments_state: Default::default(),
            error: None,
            article_type: ArticleType::Top,
            open_comments: Vec::new(),
        }
    }

    /// Handle background emitted events.
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Articles(article_type, ts) => {
                self.showing = ts.len();
                self.articles = ts;
                self.visited = Vec::new();
                self.error = None;
                self.article_type = article_type;
            }
            Event::Comments { items, parent, id } => {
                let comment_item = CommentItem {
                    comments: items,
                    parent,
                    id,
                };
                if comment_item.parent.is_some() {
                    self.comments_state.comment_trail.push(comment_item);
                    self.open_comments.push(true);
                } else {
                    // Reset comment history/state.
                    self.comments_state.comment_trail = vec![comment_item];
                    self.open_comments = vec![true];
                }
                self.error = None;
            }

            Event::Error(error) => {
                self.error = Some(error);
            }
        }
        self.fetching = false;
    }

    fn last_request(&self) -> impl Fn(usize) -> ClientEvent {
        match self.article_type {
            ArticleType::New => ClientEvent::NewStories,
            ArticleType::Best => ClientEvent::BestStories,
            ArticleType::Top => ClientEvent::TopStories,
        }
    }

    /// Handle background emitted events.
    fn handle_next_event(&mut self) {
        self.event_handler
            .next_event()
            .map(|event| self.handle_event(event))
            .unwrap_or_default();
    }

    fn render_header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("Hello").show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.style_mut().visuals.window_fill = Color32::DARK_BLUE;

                if ui
                    .selectable_label(self.article_type == ArticleType::Top, "Top")
                    .clicked()
                {
                    self.fetching = true;
                    self.event_handler
                        .emit(ClientEvent::TopStories(self.showing))
                        .unwrap_or_default();
                }
                if ui
                    .selectable_label(self.article_type == ArticleType::Best, "Best")
                    .clicked()
                {
                    self.fetching = true;
                    self.event_handler
                        .emit(ClientEvent::BestStories(self.showing))
                        .unwrap_or_default();
                }
                if ui
                    .selectable_label(self.article_type == ArticleType::New, "New")
                    .clicked()
                {
                    self.fetching = true;
                    self.event_handler
                        .emit(ClientEvent::NewStories(self.showing))
                        .unwrap_or_default();
                }

                if ui.selectable_label(self.showing == 25, "25").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(self.last_request()(25))
                        .unwrap_or_default();
                }
                if ui.selectable_label(self.showing == 50, "50").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(self.last_request()(50))
                        .unwrap_or_default();
                }
                if ui.selectable_label(self.showing == 75, "75").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(self.last_request()(75))
                        .unwrap_or_default();
                }
                if ui.selectable_label(self.showing == 100, "100").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(self.last_request()(100))
                        .unwrap_or_default();
                }
                if ui.selectable_label(self.showing == 500, "500").clicked() {
                    self.fetching = true;
                    self.event_handler
                        .emit(self.last_request()(500))
                        .unwrap_or_default();
                }
                if self.fetching {
                    ui.spinner();
                }
                if let Some(error) = self.error.as_deref() {
                    ui.label(RichText::new(error).color(Color32::RED).strong());
                }
            });
        });
    }

    /// Render the articles.
    fn render_articles(&mut self, ui: &mut egui::Ui) {
        let scroll_delta = scroll_delta(ui);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.scroll_with_delta(scroll_delta);

            Grid::new("articles")
                .num_columns(2)
                .spacing(Vec2 { x: 0., y: 5. })
                .striped(true)
                .show(ui, |ui| {
                    for (article, index) in self.articles.iter().zip(1..) {
                        ui.label(format!("{index}."));

                        ui.horizontal(|ui| {
                            if article
                                .title
                                .as_deref()
                                .unwrap_or_default()
                                .split_whitespace()
                                .any(|word| word.to_lowercase() == "rust")
                            {
                                ui.image(egui::include_image!("../rust-logo-32x32.png"));
                            }
                            if let Some(url) = article.url.as_deref() {
                                ui.style_mut().visuals.hyperlink_color =
                                    if self.visited.contains(&index) {
                                        Color32::DARK_GRAY
                                    } else {
                                        Color32::BLACK
                                    };
                                if ui
                                    .hyperlink_to(
                                        RichText::new(
                                            article.title.as_deref().unwrap_or("No title"),
                                        )
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
                                ui.style_mut().visuals.override_text_color =
                                    Some(Color32::DARK_GRAY);
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
                            ui.add_space(5.0);
                            if !article.kids.is_empty()
                                && ui.button(format!("{}", article.kids.len())).clicked()
                            {
                                // self.open_comments[0] = true;
                                self.comments_state.comments = Vec::new();
                                self.fetching = true;
                                self.comments_state.active_item = Some(article.to_owned());
                                self.visited.push(index);
                                if let Err(err) = self.event_handler.emit(ClientEvent::Comments {
                                    ids: article.kids.clone(),
                                    parent: None,
                                    id: Id::new(article.id),
                                }) {
                                    error!("Failed to emit comments: {err}");
                                }
                            }
                            ui.allocate_space(ui.available_size());
                        });

                        ui.end_row();
                    }
                });
        });
    }

    /// Render comments if requested.
    fn render_comments(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.open_comments.iter().any(|c| *c) {
            Comments {
                local_sender: &self.local_sender,
                fetching: &mut self.fetching,
                event_handler: &self.event_handler,
                // showing_comments: &mut self.showing_comments,
                open_comments: &mut self.open_comments,
                comments_state: &self.comments_state,
            }
            .render(ctx, ui);
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

        self.render_header(ctx);

        let frame = Frame {
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

            ui.add_space(2.);
            self.render_comments(ctx, ui);
            self.render_articles(ui);
        });

        // Remove comment trail for closed windows.
        self.open_comments.retain(|open| *open);
        self.comments_state
            .comment_trail
            .truncate(self.open_comments.len());
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        SHUT_DOWN.store(true, Ordering::Release);
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
