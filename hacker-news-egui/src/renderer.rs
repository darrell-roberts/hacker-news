//! Main renderer invoked on every update.
use self::styles::{
    article_text_bubble_frame, article_text_window_frame, central_panel_frame, user_bubble_frame,
    user_window_frame,
};
use crate::{
    app::{ArticleType, HackerNewsApp, MutableWidgetState},
    event::{ClientEvent, Event},
    text::{parse_date, render_rich_text},
};
use chrono::{DateTime, Utc};
use comments::Comments;
use egui::{
    include_image, style::Spacing, widgets::Widget, Align, Button, Color32, CursorIcon, Grid, Id,
    Key, Layout, RichText, TextEdit, TextStyle, Vec2, Window,
};

mod comments;
mod styles;

pub struct Renderer<'a, 'b> {
    context: &'a egui::Context,
    app_state: &'a HackerNewsApp,
    mutable_state: &'b mut MutableWidgetState,
}

impl<'a, 'b> Renderer<'a, 'b> {
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

    pub fn render(mut self) {
        if self.app_state.fetching {
            self.context.set_cursor_icon(CursorIcon::Progress);
        } else {
            self.context.set_cursor_icon(CursorIcon::Default);
        }

        self.render_header();

        egui::CentralPanel::default()
            .frame(central_panel_frame())
            .show(self.context, |ui| {
                ui.visuals_mut().widgets.noninteractive.fg_stroke.color = Color32::BLACK;
                ui.visuals_mut().widgets.active.fg_stroke.color = Color32::BLACK;
                ui.visuals_mut().widgets.hovered.fg_stroke.color = Color32::BLACK;

                ui.add_space(2.);
                self.render_comments();
                self.render_item_text();
                self.render_articles(ui);
                self.render_user();
            });
    }

    /// Render comments if requested.
    fn render_comments(&mut self) {
        Comments::new(self.context, self.app_state, self.mutable_state).render();
    }

    /// Render the articles.
    fn render_articles(&self, ui: &mut egui::Ui) {
        let scroll_delta = scroll_delta(ui);

        egui::ScrollArea::vertical()
            .id_source(Id::new(self.app_state.article_type))
            .show(ui, |ui| {
                ui.scroll_with_delta(scroll_delta);

                Grid::new("articles")
                    .num_columns(3)
                    .spacing((0., 5.))
                    .striped(true)
                    .show(ui, |ui| {
                        let article_iter = self
                            .app_state
                            .articles
                            .iter()
                            .filter(|article| {
                                if !self.app_state.search.is_empty() {
                                    article
                                        .title
                                        .as_deref()
                                        .map(|title| {
                                            title.split_whitespace().any(|word| {
                                                word.to_lowercase()
                                                    .contains(&self.app_state.search.to_lowercase())
                                            })
                                        })
                                        .unwrap_or(false)
                                } else {
                                    true
                                }
                            })
                            .filter(|article| {
                                !self.app_state.filter_visited
                                    || !self.app_state.visited.contains(&article.id)
                            });
                        for article in article_iter {
                            self.render_article(ui, article);
                        }
                    })
            });
    }

    /// Render an article.
    fn render_article(&self, ui: &mut egui::Ui, article: &hacker_news_api::Item) {
        ui.label(format!("🔼{}", article.score));

        if !article.kids.is_empty() {
            let button = Button::new(format!("💬{}", article.kids.len()))
                .fill(ui.style().visuals.window_fill())
                .ui(ui);

            if button.clicked() {
                self.app_state
                    .local_sender
                    .send(Event::FetchComments {
                        ids: article.kids.clone(),
                        parent: None,
                        id: Id::new(article.id),
                        active_item: Some(article.to_owned()),
                    })
                    .unwrap_or_default();
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

            ui.style_mut().visuals.hyperlink_color = if self.app_state.visited.contains(&article.id)
            {
                Color32::DARK_GRAY
            } else {
                Color32::BLACK
            };
            if self.app_state.visited.contains(&article.id) {
                ui.style_mut().visuals.override_text_color = Some(Color32::DARK_GRAY);
            }

            match (article.url.as_deref(), article.title.as_deref()) {
                (None, None) => (),
                (Some(_), None) => (),
                (None, Some(title)) => {
                    if ui.link(title).clicked() {
                        self.app_state
                            .local_sender
                            .send(Event::ShowItemText(article.clone()))
                            .unwrap_or_default();
                    }
                }
                (Some(url), Some(title)) => {
                    if ui
                        .hyperlink_to(RichText::new(title).strong().color(Color32::BLACK), url)
                        .clicked()
                    {
                        self.app_state
                            .local_sender
                            .send(Event::Visited(article.id))
                            .unwrap_or_default();
                    }
                }
            }

            ui.style_mut().override_text_style = Some(TextStyle::Small);
            ui.style_mut().spacing = Spacing {
                item_spacing: Vec2 { y: 1., x: 2. },
                ..Default::default()
            };

            if ui.link(RichText::new(&article.by).italics()).clicked() {
                self.app_state
                    .local_sender
                    .send(Event::FetchUser(article.by.clone()))
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

    /// Render the header.
    fn render_header(&mut self) {
        egui::TopBottomPanel::top("Hello").show(self.context, |ui| {
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
                ui.add_sized(
                    (200., 15.),
                    TextEdit::singleline(&mut self.mutable_state.search),
                );
                // ui.text_edit_singleline(&mut self.mutable_state.search);
                if ui.button("🗑").on_hover_text("Clear search").clicked() {
                    self.mutable_state.search = String::new();
                }

                ui.separator();

                ui.label(format!("{}", self.app_state.visited.len()))
                    .on_hover_text("Visited");
                let filter_button = Button::image(include_image!("../assets/filter.png"))
                    .selected(self.app_state.filter_visited);
                if filter_button
                    .ui(ui)
                    .on_hover_text("Filter visited")
                    .clicked()
                {
                    self.app_state
                        .local_sender
                        .send(Event::FilterVisited)
                        .unwrap_or_default();
                }
                let reset_button = Button::image(include_image!("../assets/reset.png"));
                if reset_button.ui(ui).on_hover_text("Reset visited").clicked() {
                    self.app_state
                        .local_sender
                        .send(Event::ResetVisited)
                        .unwrap_or_default();
                };

                if self.app_state.fetching {
                    ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                        ui.spinner();
                    });
                }
            });

            if let Some(error) = self.app_state.error.as_deref() {
                ui.label(RichText::new(error).color(Color32::RED).strong());
            }
        });
    }

    fn add_total_select_label(&self, ui: &mut egui::Ui, total: usize) {
        if ui
            .selectable_label(self.app_state.showing == total, format!("{total}"))
            .clicked()
        {
            self.app_state
                .local_sender
                .send(Event::FetchArticles(self.app_state.last_request()(total)))
                .unwrap_or_default();
        }
    }

    fn add_article_type_selet_label(&self, ui: &mut egui::Ui, article_type: ArticleType) {
        if ui
            .selectable_label(
                self.app_state.article_type == article_type,
                article_type.as_str(),
            )
            .clicked()
        {
            self.app_state
                .local_sender
                .send(Event::FetchArticles(match article_type {
                    ArticleType::New => ClientEvent::NewStories(self.app_state.showing),
                    ArticleType::Best => ClientEvent::BestStories(self.app_state.showing),
                    ArticleType::Top => ClientEvent::TopStories(self.app_state.showing),
                }))
                .unwrap_or_default();
        }
    }

    /// Render a user window.
    fn render_user(&mut self) {
        if let Some(user) = self.app_state.user.as_ref() {
            Window::new(&user.id)
                .open(&mut self.mutable_state.viewing_user)
                .frame(user_window_frame())
                .collapsible(false)
                .vscroll(true)
                .show(self.context, |ui| {
                    if let Some(about) = user.about.as_deref() {
                        user_bubble_frame().show(ui, |ui| {
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

    /// Render an item text window.
    fn render_item_text(&mut self) {
        if let Some(item) = self.app_state.comments_state.active_item.as_ref() {
            if self.app_state.viewing_item_text {
                egui::Window::new("")
                    .id(Id::new(item.id))
                    .frame(article_text_window_frame())
                    .vscroll(true)
                    .collapsible(false)
                    .open(&mut self.mutable_state.viewing_item_text)
                    .show(self.context, |ui| {
                        article_text_bubble_frame().show(ui, |ui| {
                            render_rich_text(item.text.as_deref().unwrap_or_default(), ui);
                        });
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
                        });
                    });
            }
        }
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
