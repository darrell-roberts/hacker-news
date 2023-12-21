use self::comments::CommentItem;
use crate::{
    event::{ClientEvent, Event, EventHandler},
    text::{parse_date, render_rich_text},
    SHUT_DOWN,
};
use chrono::{DateTime, Utc};
use comments::{Comments, CommentsState};
use egui::{
    epaint::Shadow, os::OperatingSystem, style::Spacing, widgets::Widget, Button, Color32,
    CursorIcon, Frame, Grid, Id, Key, Margin, RichText, Rounding, Stroke, TextStyle, Vec2, Window,
};
use hacker_news_api::{Item, User};
use log::error;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc::UnboundedSender;

mod comments;

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum ArticleType {
    New,
    Best,
    Top,
}

impl ArticleType {
    fn as_str(&self) -> &str {
        match self {
            ArticleType::New => "New",
            ArticleType::Best => "Best",
            ArticleType::Top => "Top",
        }
    }
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
    visited: Vec<u64>,
    /// Comments state.
    comments_state: CommentsState,
    /// Errors.
    error: Option<String>,
    /// Viewing article type.
    article_type: ArticleType,
    /// Comment window open states.
    open_comments: Vec<bool>,
    /// Viewing a user
    user: Option<User>,
    /// User window open/closed.
    viewing_user: bool,
    /// Search input.
    search: String,
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
            user: None,
            viewing_user: false,
            search: String::new(),
        }
    }

    /// Handle background emitted events.
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Articles(article_type, ts) => {
                self.showing = ts.len();
                self.articles = ts;
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
            Event::User(user) => {
                self.viewing_user = true;
                self.user = Some(user);
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

    fn add_total_select_label(&mut self, ui: &mut egui::Ui, total: usize) {
        if ui
            .selectable_label(self.showing == total, format!("{total}"))
            .clicked()
        {
            self.fetching = true;
            self.event_handler
                .emit(self.last_request()(total))
                .unwrap_or_default();
        }
    }

    fn add_article_type_selet_label(&mut self, ui: &mut egui::Ui, article_type: ArticleType) {
        if ui
            .selectable_label(self.article_type == article_type, article_type.as_str())
            .clicked()
        {
            self.fetching = true;
            self.event_handler
                .emit(match article_type {
                    ArticleType::New => ClientEvent::NewStories(self.showing),
                    ArticleType::Best => ClientEvent::BestStories(self.showing),
                    ArticleType::Top => ClientEvent::TopStories(self.showing),
                })
                .unwrap_or_default();
        }
    }

    fn render_header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("Hello").show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.style_mut().visuals.window_fill = Color32::DARK_BLUE;

                self.add_article_type_selet_label(ui, ArticleType::Top);
                self.add_article_type_selet_label(ui, ArticleType::Best);
                self.add_article_type_selet_label(ui, ArticleType::New);

                ui.separator();

                self.add_total_select_label(ui, 25);
                self.add_total_select_label(ui, 50);
                self.add_total_select_label(ui, 75);
                self.add_total_select_label(ui, 100);
                self.add_total_select_label(ui, 500);

                ui.separator();

                ui.label("🔎");
                ui.text_edit_singleline(&mut self.search);
                if ui.button("🗑").on_hover_text("Clear search").clicked() {
                    self.search = String::new();
                }

                ui.separator();

                if self.fetching {
                    ui.spinner();
                }
            });

            if let Some(error) = self.error.as_deref() {
                ui.label(RichText::new(error).color(Color32::RED).strong());
            }
        });
    }

    /// Render the articles.
    fn render_articles(&mut self, ui: &mut egui::Ui) {
        let scroll_delta = scroll_delta(ui);

        egui::ScrollArea::vertical()
            .id_source(Id::new(self.article_type))
            .show(ui, |ui| {
                ui.scroll_with_delta(scroll_delta);

                Grid::new("articles")
                    .num_columns(3)
                    .spacing((0., 5.))
                    .striped(true)
                    .show(ui, |ui| {
                        for article in self.articles.iter().filter(|article| {
                            if !self.search.is_empty() {
                                article
                                    .title
                                    .as_deref()
                                    .map(|title| {
                                        title.split_whitespace().any(|word| {
                                            word.to_lowercase()
                                                .contains(&self.search.to_lowercase())
                                        })
                                    })
                                    .unwrap_or(false)
                            } else {
                                true
                            }
                        }) {
                            ui.label(format!("🔼{}", article.score));

                            if !article.kids.is_empty() {
                                let button = Button::new(format!("💬{}", article.kids.len()))
                                    .fill(ui.style().visuals.window_fill())
                                    .ui(ui);

                                if button.clicked() {
                                    self.comments_state.comments = Vec::new();
                                    self.fetching = true;
                                    self.comments_state.active_item = Some(article.to_owned());
                                    self.visited.push(article.id);
                                    if let Err(err) =
                                        self.event_handler.emit(ClientEvent::Comments {
                                            ids: article.kids.clone(),
                                            parent: None,
                                            id: Id::new(article.id),
                                        })
                                    {
                                        error!("Failed to emit comments: {err}");
                                    }
                                }
                            } else {
                                ui.label("");
                            }

                            ui.horizontal(|ui| {
                                // Add rust icon.
                                if article
                                    .title
                                    .as_deref()
                                    .unwrap_or_default()
                                    .split_whitespace()
                                    .any(|word| word.to_lowercase() == "rust")
                                {
                                    ui.image(egui::include_image!("../assets/rust-logo-32x32.png"));
                                }

                                ui.style_mut().visuals.hyperlink_color =
                                    if self.visited.contains(&article.id) {
                                        Color32::DARK_GRAY
                                    } else {
                                        Color32::BLACK
                                    };
                                if self.visited.contains(&article.id) {
                                    ui.style_mut().visuals.override_text_color =
                                        Some(Color32::DARK_GRAY);
                                }

                                match (article.url.as_deref(), article.title.as_deref()) {
                                    (None, None) => (),
                                    (Some(_), None) => (),
                                    (None, Some(title)) => {
                                        if ui.link(title).clicked() {
                                            //Render comment.
                                            self.comments_state.active_item =
                                                Some(article.to_owned());
                                            self.visited.push(article.id);
                                            self.local_sender
                                                .send(Event::Comments {
                                                    items: Vec::new(),
                                                    parent: None,
                                                    id: Id::new(article.id),
                                                })
                                                .unwrap_or_default();
                                        }
                                    }
                                    (Some(url), Some(title)) => {
                                        if ui
                                            .hyperlink_to(
                                                RichText::new(title).strong().color(Color32::BLACK),
                                                url,
                                            )
                                            .clicked()
                                        {
                                            self.visited.push(article.id);
                                        }
                                    }
                                }

                                ui.style_mut().override_text_style = Some(TextStyle::Small);
                                ui.style_mut().spacing = Spacing {
                                    item_spacing: Vec2 { y: 1., x: 2. },
                                    ..Default::default()
                                };

                                if ui.link(RichText::new(&article.by).italics()).clicked() {
                                    self.fetching = true;
                                    self.event_handler
                                        .emit(ClientEvent::User(article.by.clone()))
                                        .unwrap_or_default();
                                };
                                if let Some(time) = parse_date(article.time) {
                                    ui.label(RichText::new(time).italics());
                                }
                                ui.add_space(5.0);

                                ui.allocate_space(ui.available_size());
                            });

                            ui.end_row();
                        }
                    })
            });
    }

    /// Render comments if requested.
    fn render_comments(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.open_comments.iter().any(|c| *c) {
            Comments {
                local_sender: &self.local_sender,
                fetching: &mut self.fetching,
                event_handler: &self.event_handler,
                open_comments: &mut self.open_comments,
                comments_state: &self.comments_state,
            }
            .render(ctx, ui);
        }
    }

    fn render_user(&mut self, ctx: &egui::Context) {
        if let Some(user) = self.user.as_ref() {
            let frame = Frame::none()
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
                .shadow(Shadow::small_light())
                .fill(Color32::from_rgb(220, 245, 247));
            Window::new(&user.id)
                .open(&mut self.viewing_user)
                .frame(frame)
                .collapsible(false)
                .vscroll(true)
                .show(ctx, |ui| {
                    if let Some(about) = user.about.as_deref() {
                        Frame::none()
                            .fill(Color32::LIGHT_BLUE)
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
                                render_rich_text(about, ui);
                            });
                    }

                    let created = DateTime::<Utc>::from_timestamp(user.created as i64, 0);
                    ui.add_space(5.);
                    ui.horizontal(|ui| {
                        match created {
                            Some(c) => {
                                ui.label(format!("Registered: {}", c.format("%d/%m/%Y")));
                            }
                            None => {
                                ui.label("No registration date");
                            }
                        };

                        ui.label(format!("karma: {}", user.karma));
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
            self.render_user(ctx);
        });
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
