//! Main renderer invoked on every update.
use self::styles::{
    article_text_bubble_frame, article_text_window_frame, central_panel_frame, user_bubble_frame,
    user_window_frame,
};
use crate::{
    app::{Filter, HackerNewsApp, MutableWidgetState},
    event::{ClientEvent, Event},
};
use chrono::{DateTime, Utc};
use egui::{
    include_image, style::Spacing, widgets::Widget, Align, Button, Color32, CursorIcon, Grid, Id,
    Key, Layout, RichText, TextStyle, Vec2, Window,
};
use hacker_news_api::{ArticleType, Item, ResultExt};

mod comments;
mod styles;
mod text;

/// Render the central panel and all child widgets.
pub fn render<'a>(
    context: &'a egui::Context,
    app_state: &'a HackerNewsApp,
    mutable_state: &mut MutableWidgetState,
) {
    if app_state.fetching {
        context.set_cursor_icon(CursorIcon::Progress);
    } else {
        context.set_cursor_icon(CursorIcon::Default);
    }

    render_header(context, app_state, mutable_state);
    render_footer(context, app_state);

    egui::CentralPanel::default()
        .frame(central_panel_frame())
        .show(context, |ui| {
            ui.visuals_mut().widgets.noninteractive.fg_stroke.color = Color32::BLACK;
            ui.visuals_mut().widgets.active.fg_stroke.color = Color32::BLACK;
            ui.visuals_mut().widgets.hovered.fg_stroke.color = Color32::BLACK;

            ui.add_space(2.);
            comments::render(context, app_state, mutable_state);
            render_item_text(context, app_state, mutable_state);
            render_articles(app_state, ui);
            render_user(context, app_state, mutable_state);
        });
}

/// Render the articles.
fn render_articles(app_state: &HackerNewsApp, ui: &mut egui::Ui) {
    let scroll_delta = scroll_delta(ui);

    egui::ScrollArea::vertical()
        .id_source(Id::new(app_state.article_type))
        .show(ui, |ui| {
            ui.scroll_with_delta(scroll_delta);

            Grid::new("articles")
                .num_columns(if app_state.article_type == ArticleType::Job {
                    1
                } else {
                    3
                })
                .spacing((0., 5.))
                .striped(true)
                .show(ui, |ui| {
                    let article_iter = app_state
                        .articles
                        .iter()
                        .filter(|article| {
                            if !app_state.search.is_empty() {
                                article
                                    .title
                                    .as_deref()
                                    .map(|title| {
                                        title.split_whitespace().any(|word| {
                                            word.to_lowercase()
                                                .contains(&app_state.search.to_lowercase())
                                        })
                                    })
                                    .unwrap_or(false)
                            } else {
                                true
                            }
                        })
                        .filter(|article| {
                            !app_state.filters.contains(&crate::app::Filter::Visisted)
                                || !app_state.visited.contains(&article.id)
                        })
                        .filter(|article| {
                            !app_state.filters.iter().any(|&f| {
                                f == Filter::Jobs || f == Filter::Polls || f == Filter::Stories
                            }) || match article.ty.as_str() {
                                "story" => app_state.filters.contains(&Filter::Stories),
                                "job" => app_state.filters.contains(&Filter::Jobs),
                                "poll" => app_state.filters.contains(&Filter::Polls),
                                _ => false,
                            }
                        });

                    article_iter.for_each(render_article(app_state, ui));
                })
        });
}

/// Render an article.
fn render_article<'a: 'b, 'b>(
    app_state: &'a HackerNewsApp,
    ui: &'b mut egui::Ui,
) -> impl FnMut(&'b Item) + 'b {
    |article| {
        // No comments / score for Job view so we remove these columns
        if app_state.article_type != ArticleType::Job {
            ui.label(format!("🔼{}", article.score));

            if !article.kids.is_empty() {
                let button = Button::new(format!("💬{}", article.kids.len()))
                    .fill(ui.style().visuals.window_fill())
                    .ui(ui);

                if button.clicked() {
                    app_state
                        .local_sender
                        .send(Event::FetchComments {
                            ids: article.kids.clone(),
                            parent: None,
                            id: Id::new(article.id),
                            active_item: Some(article.to_owned()),
                        })
                        .log_error_consume();
                }
            } else {
                ui.label("");
            }
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

            // Add job icon.
            if article.ty == "job" {
                ui.label("🏢");
            }

            ui.style_mut().visuals.hyperlink_color = if app_state.visited.contains(&article.id) {
                Color32::DARK_GRAY
            } else {
                Color32::BLACK
            };
            if app_state.visited.contains(&article.id) {
                ui.style_mut().visuals.override_text_color = Some(Color32::DARK_GRAY);
            }

            match (article.url.as_deref(), article.title.as_deref()) {
                (None, Some(title)) => {
                    if ui.link(title).clicked() {
                        app_state
                            .local_sender
                            .send(Event::ShowItemText(article.clone()))
                            .log_error_consume();
                    }
                }
                (Some(url), Some(title)) => {
                    if ui
                        .hyperlink_to(RichText::new(title).strong().color(Color32::BLACK), url)
                        .clicked()
                    {
                        app_state
                            .local_sender
                            .send(Event::Visited(article.id))
                            .log_error_consume();
                    }
                }
                _ => (),
            }

            ui.style_mut().override_text_style = Some(TextStyle::Small);
            ui.style_mut().spacing = Spacing {
                item_spacing: Vec2 { y: 1., x: 2. },
                ..Default::default()
            };

            if ui.link(RichText::new(&article.by).italics()).clicked() {
                app_state
                    .local_sender
                    .send(Event::FetchUser(article.by.clone()))
                    .log_error_consume();
            };
            if let Some(time) = text::parse_date(article.time) {
                ui.label(RichText::new(time).italics());
            }
            ui.allocate_space(ui.available_size());
        });
        ui.end_row();
    }
}

/// Render the header.
fn render_header<'a>(
    context: &'a egui::Context,
    app_state: &'a HackerNewsApp,
    mutable_state: &mut MutableWidgetState,
) {
    egui::TopBottomPanel::top("header").show(context, |ui| {
        // Header
        ui.horizontal(|ui| {
            ui.style_mut().visuals.window_fill = Color32::DARK_BLUE;

            [ArticleType::Top, ArticleType::Best, ArticleType::New]
                .into_iter()
                .for_each(add_article_type_select_label(app_state, ui));

            ui.separator();

            [ArticleType::Ask, ArticleType::Show, ArticleType::Job]
                .into_iter()
                .for_each(add_article_type_select_label(app_state, ui));

            ui.separator();

            [25, 50, 75, 100, 500]
                .into_iter()
                .for_each(add_total_select_label(app_state, ui));

            ui.separator();

            if Button::new("🔎")
                .selected(app_state.search_open)
                .ui(ui)
                .on_hover_text("Open search")
                .clicked()
            {
                app_state
                    .local_sender
                    .send(Event::ToggleOpenSearch)
                    .log_error_consume();
            }
            ui.separator();

            ui.label(format!("{}", app_state.visited.len()))
                .on_hover_text("Visited");
            let filter_button = Button::image(include_image!("../assets/filter.png"))
                .selected(app_state.filters.contains(&Filter::Visisted));
            if filter_button
                .ui(ui)
                .on_hover_text("Filter visited")
                .clicked()
            {
                app_state
                    .local_sender
                    .send(Event::ToggleFilter(Filter::Visisted))
                    .log_error_consume();
            }
            let reset_button = Button::image(include_image!("../assets/reset.png"));
            if reset_button.ui(ui).on_hover_text("Reset visited").clicked() {
                app_state
                    .local_sender
                    .send(Event::ResetVisited)
                    .log_error_consume();
            };
        });

        if app_state.search_open {
            ui.horizontal(|ui| {
                ui.label("🔎");
                ui.text_edit_singleline(&mut mutable_state.search)
                    .request_focus();

                if ui.button("🗑").on_hover_text("Clear search").clicked() {
                    mutable_state.search = String::new();
                }
            });
        }

        if let Some(error) = app_state.error.as_deref() {
            ui.label(RichText::new(error).color(Color32::RED).strong());
        }
    });
}

fn add_total_select_label<'a, 'b: 'a>(
    app_state: &'a HackerNewsApp,
    ui: &'b mut egui::Ui,
) -> impl FnMut(usize) + 'a {
    |total| {
        if ui
            .selectable_label(app_state.showing == total, format!("{total}"))
            .clicked()
        {
            app_state
                .local_sender
                .send(Event::FetchArticles(app_state.last_request()(total)))
                .log_error_consume();
        }
    }
}

fn add_article_type_select_label<'a, 'b: 'a>(
    app_state: &'b HackerNewsApp,
    ui: &'a mut egui::Ui,
) -> impl FnMut(ArticleType) + 'a {
    |article_type: ArticleType| {
        if ui
            .selectable_label(
                app_state.article_type == article_type,
                article_type.as_str(),
            )
            .clicked()
        {
            app_state
                .local_sender
                .send(Event::FetchArticles(match article_type {
                    ArticleType::New => ClientEvent::NewStories(app_state.showing),
                    ArticleType::Best => ClientEvent::BestStories(app_state.showing),
                    ArticleType::Top => ClientEvent::TopStories(app_state.showing),
                    ArticleType::Ask => ClientEvent::AskStories(app_state.showing),
                    ArticleType::Show => ClientEvent::ShowStories(app_state.showing),
                    ArticleType::Job => ClientEvent::JobStories(app_state.showing),
                }))
                .log_error_consume();
        }
    }
}

/// Render a user window.
fn render_user<'a>(
    context: &'a egui::Context,
    app_state: &'a HackerNewsApp,
    mutable_state: &mut MutableWidgetState,
) {
    if let Some(user) = app_state.user.as_ref() {
        Window::new(&user.id)
            .open(&mut mutable_state.viewing_user)
            .frame(user_window_frame())
            .collapsible(false)
            .vscroll(true)
            .show(context, |ui| {
                if let Some(about) = user.about.as_deref() {
                    user_bubble_frame().show(ui, |ui| {
                        text::render_rich_text(about, ui);
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
fn render_item_text<'a>(
    context: &'a egui::Context,
    app_state: &'a HackerNewsApp,
    mutable_state: &mut MutableWidgetState,
) {
    if let Some(item) = app_state.comments_state.active_item.as_ref() {
        if app_state.viewing_item_text {
            egui::Window::new("")
                .id(Id::new(item.id))
                .frame(article_text_window_frame())
                .vscroll(true)
                .collapsible(false)
                .open(&mut mutable_state.viewing_item_text)
                .show(context, |ui| {
                    article_text_bubble_frame().show(ui, |ui| {
                        text::render_rich_text(item.text.as_deref().unwrap_or_default(), ui);
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
                            app_state
                                .local_sender
                                .send(Event::FetchUser(item.by.clone()))
                                .log_error_consume();
                        };

                        if let Some(time) = text::parse_date(item.time) {
                            ui.label(RichText::new(time).italics());
                        }
                    });
                });
        }
    }
}

// Key bindings that change scrolling position.
fn scroll_delta(ui: &egui::Ui) -> Vec2 {
    let mut scroll_delta = Vec2::ZERO;
    ui.input(|input| {
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

fn render_footer<'a>(context: &'a egui::Context, app_state: &'a HackerNewsApp) {
    egui::TopBottomPanel::bottom("bottom").show(context, |ui| {
        ui.horizontal(|ui| {
            if let Some(dt) = app_state.last_update.as_ref() {
                ui.label(format!("Updated: {}", dt.format("%d/%m/%Y %r")));
            }

            if app_state.fetching {
                ui.spinner();
            }

            ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                let (stories, polls, jobs) =
                    app_state
                        .articles
                        .iter()
                        .fold((0, 0, 0), |(stories, polls, jobs), item| {
                            match item.ty.as_str() {
                                "job" => (stories, polls, jobs + 1),
                                "story" => (stories + 1, polls, jobs),
                                "poll" => (stories, polls + 1, jobs),
                                _ => (stories, polls, jobs),
                            }
                        });

                let mut filter_button = |name, hover_text, filter| {
                    if Button::new(name)
                        .selected(app_state.filters.contains(&filter))
                        .ui(ui)
                        .on_hover_text(hover_text)
                        .clicked()
                    {
                        // dispatch filter change
                        app_state
                            .local_sender
                            .send(Event::ToggleFilter(filter))
                            .log_error_consume();
                    };
                };

                filter_button(
                    format!("stories: {stories}"),
                    "Filter stories",
                    Filter::Stories,
                );
                filter_button(format!("polls: {polls}"), "Filter polls", Filter::Polls);
                filter_button(format!("jobs: {jobs}"), "Filter jobs", Filter::Jobs);
            });
        });
    });
}
