use crate::comment::{CommentItem, CommentState};
use anyhow::Result;
use chrono::{DateTime, Local};
use hacker_news_api::{ApiClient, ArticleType, Item};
use iced::{
    widget::{self, column, container},
    Element, Task, Theme,
};
use log::error;
use std::{collections::HashSet, sync::Arc};

/// The current state of what we are showing.
pub struct Showing {
    /// Limit of articles
    pub limit: usize,
    /// Type of article
    pub article_type: ArticleType,
}

/// Application state.
pub struct App {
    /// Viewing articles
    pub articles: Vec<Item>,
    /// API Client.
    pub client: Arc<ApiClient>,
    /// What are we showing
    pub showing: Showing,
    /// Status line message
    pub status_line: String,
    /// Comments being viewed.
    pub comments: Option<CommentState>,
    /// Visisted item ids.
    pub visited: HashSet<u64>,
    /// Active theme.
    pub theme: Theme,
    /// Search
    pub search: Option<String>,
    /// All articles for search.
    pub all_articles: Vec<Item>,
    /// Scale.
    pub scale: f64,
    /// Last update
    pub last_update: Option<DateTime<Local>>,
}

#[derive(Debug)]
pub enum AppMsg {
    Fetch {
        limit: usize,
        article_type: ArticleType,
    },
    Receive(Result<Vec<Item>>),
    OpenComment {
        article: Option<Item>,
        comment_ids: Vec<u64>,
        parent: Option<Item>,
    },
    ReceiveComments(Result<Vec<Item>>, Option<Item>),
    CloseComment,
    OpenLink {
        url: String,
        item_id: u64,
    },
    ChangeTheme(Theme),
    OpenSearch,
    CloseSearch,
    Search(String),
    WindowClose,
    IncreaseScale,
    DecreaseScale,
    ResetScale,
    Url(String),
    NoUrl,
}

impl Clone for AppMsg {
    fn clone(&self) -> Self {
        match self {
            &AppMsg::Fetch {
                limit,
                article_type,
            } => Self::Fetch {
                limit,
                article_type,
            },
            AppMsg::Receive(_) => unimplemented!("Receive is not cloned"),
            AppMsg::ReceiveComments(_, _) => unimplemented!("Receive is not cloned"),
            AppMsg::OpenComment {
                article,
                comment_ids,
                parent,
            } => AppMsg::OpenComment {
                article: article.clone(),
                comment_ids: comment_ids.clone(),
                parent: parent.clone(),
            },
            AppMsg::CloseComment => AppMsg::CloseComment,
            AppMsg::OpenLink { url, item_id } => AppMsg::OpenLink {
                url: url.clone(),
                item_id: *item_id,
            },
            AppMsg::ChangeTheme(theme) => AppMsg::ChangeTheme(theme.clone()),
            AppMsg::OpenSearch => AppMsg::OpenSearch,
            AppMsg::CloseSearch => AppMsg::CloseSearch,
            AppMsg::Search(s) => AppMsg::Search(s.clone()),
            AppMsg::WindowClose => AppMsg::WindowClose,
            AppMsg::DecreaseScale => AppMsg::DecreaseScale,
            AppMsg::IncreaseScale => AppMsg::IncreaseScale,
            AppMsg::ResetScale => AppMsg::ResetScale,
            AppMsg::Url(s) => AppMsg::Url(s.clone()),
            AppMsg::NoUrl => AppMsg::NoUrl,
        }
    }
}

pub(crate) fn update(app: &mut App, message: AppMsg) -> Task<AppMsg> {
    match message {
        AppMsg::Fetch {
            limit,
            article_type,
        } => {
            let client = app.client.clone();
            app.showing.limit = limit;
            app.showing.article_type = article_type;
            app.status_line = "Fetching...".into();
            Task::perform(
                async move { client.articles(limit, article_type).await },
                AppMsg::Receive,
            )
        }
        AppMsg::Receive(items) => match items {
            Ok(articles) => {
                app.articles = articles;
                let dt = Local::now();
                app.status_line = format!("Updated: {}", dt.format("%d/%m/%Y %r"));
                app.last_update = Some(dt);
                widget::scrollable::scroll_to(
                    widget::scrollable::Id::new("articles"),
                    Default::default(),
                )
            }
            Err(err) => {
                app.status_line = err.to_string();
                Task::none()
            }
        },
        AppMsg::OpenComment {
            article,
            comment_ids,
            parent,
        } => {
            app.status_line = "Fetching...".into();
            // Opening first set of comments from an article.
            if let Some(item) = article {
                app.visited.insert(item.id);
                app.comments = Some(CommentState {
                    article: item,
                    comments: Vec::new(),
                })
            }

            let client = app.client.clone();
            Task::perform(
                async move { client.items(&comment_ids).await },
                move |result| AppMsg::ReceiveComments(result, parent.clone()),
            )
        }
        AppMsg::ReceiveComments(result, parent) => {
            match result {
                Ok(comments) => {
                    // app.status_line = format!("Updated: {}", Local::now().format("%d/%m/%Y %r"));
                    match app.last_update.as_ref() {
                        Some(dt) => {
                            app.status_line = format!("Updated: {}", dt.format("%d/%m/%Y %r"))
                        }
                        None => app.status_line.clear(),
                    }

                    if let Some(stack) = app.comments.as_mut() {
                        stack.comments.push(CommentItem {
                            items: comments,
                            parent,
                        });
                    }
                }
                Err(err) => {
                    app.status_line = err.to_string();
                }
            }
            Task::none()
        }
        AppMsg::CloseComment => {
            if let Some(comment_stack) = app.comments.as_mut() {
                comment_stack.comments.pop();
                if comment_stack.comments.is_empty() {
                    app.comments = None;
                }
            }
            Task::none()
        }
        AppMsg::OpenLink { url, item_id } => {
            app.visited.insert(item_id);
            open::with(url, "firefox")
                .inspect_err(|err| {
                    error!("Failed to open url {err}");
                })
                .unwrap_or_default();
            Task::none()
        }
        AppMsg::ChangeTheme(theme) => {
            app.theme = theme;
            Task::none()
        }
        AppMsg::OpenSearch => {
            app.search = Some(String::new());
            app.all_articles = app.articles.clone();
            widget::text_input::focus(widget::text_input::Id::new("search"))
        }
        AppMsg::CloseSearch => {
            if app.search.is_some() {
                app.search = None;
                app.articles = std::mem::take(&mut app.all_articles);
            }
            Task::none()
        }
        AppMsg::Search(input) => {
            app.articles = app
                .all_articles
                .iter()
                .filter(|item| {
                    item.title
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&input.to_lowercase()))
                        .unwrap_or(false)
                })
                .map(ToOwned::to_owned)
                .collect();
            app.search.replace(input);
            Task::none()
        }
        AppMsg::WindowClose => {
            println!("Window close event");
            Task::none()
        }
        AppMsg::IncreaseScale => {
            app.scale += 0.1;
            Task::none()
        }
        AppMsg::DecreaseScale => {
            let new_scale = app.scale - 0.1;
            let int = new_scale * 100.0;

            if int > 10.0 {
                app.scale = new_scale;
            }
            Task::none()
        }
        AppMsg::ResetScale => {
            app.scale = 1.0;
            Task::none()
        }
        AppMsg::Url(url) => {
            if app.status_line != url {
                app.status_line = url;
            }
            Task::none()
        }
        AppMsg::NoUrl => {
            match app.last_update.as_ref() {
                Some(dt) => app.status_line = format!("Updated: {}", dt.format("%d/%m/%Y %r")),
                None => app.status_line.clear(),
            }
            Task::none()
        }
    }
}

pub(crate) fn view(app: &App) -> iced::Element<AppMsg> {
    let content = match app.comments.as_ref() {
        Some(comments) => {
            Element::from(column![app.render_comments(comments), app.render_footer()])
        }
        None => {
            let col = column![
                app.render_header(),
                app.render_articles(),
                app.render_footer()
            ];
            Element::from(col.spacing(10.))
        }
    };

    container(content).into()
}
