use crate::comment::{CommentItem, CommentState};
use anyhow::Result;
use chrono::Local;
use hacker_news_api::{ApiClient, ArticleType, Item};
use iced::{
    alignment::Vertical,
    widget::{column, container, text},
    Element, Task,
};
use log::error;
use std::sync::Arc;

pub struct Showing {
    pub limit: usize,
    pub article_type: ArticleType,
}

pub struct App {
    pub articles: Vec<Item>,
    pub client: Arc<ApiClient>,
    pub showing: Showing,
    pub status_line: String,
    pub comments: Option<CommentState>,
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
    OpenLink(String),
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
            AppMsg::OpenLink(url) => AppMsg::OpenLink(url.clone()),
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
        AppMsg::Receive(items) => {
            match items {
                Ok(articles) => {
                    app.articles = articles;
                    let dt = Local::now();
                    app.status_line = format!("Updated: {}", dt.format("%d/%m/%Y %r"));
                }
                Err(err) => {
                    app.status_line = err.to_string();
                }
            }
            Task::none()
        }
        AppMsg::OpenComment {
            article,
            comment_ids,
            parent,
        } => {
            // Opening first set of comments from an article.
            if let Some(item) = article {
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
                Ok(comments) => match app.comments.as_mut() {
                    Some(stack) => {
                        stack.comments.push(CommentItem {
                            items: comments,
                            parent,
                        });
                    }
                    None => unreachable!(),
                },
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
        AppMsg::OpenLink(url) => {
            open::that(url)
                .inspect_err(|err| {
                    error!("Failed to open url {err}");
                })
                .unwrap_or_default();
            Task::none()
        }
    }
}

pub(crate) fn view(app: &App) -> iced::Element<AppMsg> {
    let content = match app.comments.as_ref() {
        Some(comments) => app.render_comments(comments),
        None => {
            let col = column![
                container(app.render_header()).padding([10, 0]),
                app.render_articles(),
                container(text(&app.status_line))
                    .align_y(Vertical::Bottom)
                    .padding([0, 10])
            ];
            Element::from(col.spacing(10.))
        }
    };

    container(content).into()
}
