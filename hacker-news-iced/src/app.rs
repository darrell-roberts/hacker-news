use crate::comment::{CommentItem, CommentState};
use anyhow::Result;
use chrono::Local;
use hacker_news_api::{ApiClient, ArticleType, Item};
use iced::{
    alignment::Vertical,
    executor,
    widget::{column, container, text},
    Application, Command, Element, Theme,
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

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = AppMsg;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let client = Arc::new(ApiClient::new().expect("Valid client"));
        (
            Self {
                articles: Vec::new(),
                client: client.clone(),
                showing: Showing {
                    limit: 50,
                    article_type: ArticleType::Top,
                },
                status_line: String::new(),
                comments: None,
            },
            iced::Command::perform(
                async move { client.articles(20, ArticleType::Top).await },
                AppMsg::Receive,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("Hacker News")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMsg::Fetch {
                limit,
                article_type,
            } => {
                let client = self.client.clone();
                self.showing.limit = limit;
                self.showing.article_type = article_type;
                self.status_line = "Fetching...".into();
                return Command::perform(
                    async move { client.articles(limit, article_type).await },
                    AppMsg::Receive,
                );
            }
            AppMsg::Receive(items) => match items {
                Ok(articles) => {
                    self.articles = articles;
                    let dt = Local::now();
                    self.status_line = format!("Updated: {}", dt.format("%d/%m/%Y %r"));
                }
                Err(err) => {
                    self.status_line = err.to_string();
                }
            },
            AppMsg::OpenComment {
                article,
                comment_ids,
                parent,
            } => {
                // Opening first set of comments from an article.
                if let Some(item) = article {
                    self.comments = Some(CommentState {
                        article: item,
                        comments: Vec::new(),
                    })
                }

                let client = self.client.clone();
                return Command::perform(
                    async move { client.items(&comment_ids).await },
                    |result| AppMsg::ReceiveComments(result, parent),
                );
            }
            AppMsg::ReceiveComments(result, parent) => match result {
                Ok(comments) => match self.comments.as_mut() {
                    Some(stack) => {
                        stack.comments.push(CommentItem {
                            items: comments,
                            parent,
                        });
                    }
                    None => unreachable!(),
                },
                Err(err) => {
                    self.status_line = err.to_string();
                }
            },
            AppMsg::CloseComment => {
                if let Some(comment_stack) = self.comments.as_mut() {
                    comment_stack.comments.pop();
                    if comment_stack.comments.is_empty() {
                        self.comments = None;
                    }
                }
            }
            AppMsg::OpenLink(url) => open::that(url)
                .inspect_err(|err| {
                    error!("Failed to open url {err}");
                })
                .unwrap_or_default(),
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let content = match self.comments.as_ref() {
            Some(comments) => self.render_comments(comments),
            None => {
                let col = column![
                    container(self.render_header()).padding([10, 0, 0, 0]),
                    self.render_articles(),
                    container(text(&self.status_line))
                        .align_y(Vertical::Bottom)
                        .padding([0, 10, 0, 10])
                ];
                Element::from(col.spacing(10.))
            }
        };

        container(content).into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::GruvboxDark
    }
}
