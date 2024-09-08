use crate::{
    articles::{self, ArticleMsg, ArticleState},
    comment::{self, CommentMsg, CommentState},
    footer::{self, FooterMsg, FooterState},
    header::{self, HeaderState},
};
use hacker_news_api::{ApiClient, ArticleType, Item};
use iced::{
    widget::{column, container},
    Element, Task, Theme,
};
use log::error;
use std::sync::Arc;

/// Application state.
pub struct App {
    /// Active theme.
    pub theme: Theme,
    /// Scale.
    pub scale: f64,
    /// Header
    pub header: HeaderState,
    /// Main content
    pub content: ContentScreen,
    /// Footer
    pub footer: FooterState,
    /// API Client.
    pub client: Arc<ApiClient>,
    /// Article state when viewing comments.
    pub article_state: Option<ArticleState>,
}

pub enum ContentScreen {
    Articles(ArticleState),
    Comments(CommentState),
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    Header(header::HeaderMsg),
    Articles(articles::ArticleMsg),
    Footer(footer::FooterMsg),
    Comments(comment::CommentMsg),
    OpenComment {
        article: Option<Item>,
        comment_ids: Vec<u64>,
        parent: Option<Item>,
    },
    OpenLink {
        url: String,
        item_id: u64,
    },
    ChangeTheme(Theme),
    WindowClose,
    IncreaseScale,
    DecreaseScale,
    ResetScale,
    RestoreArticles,
}

pub(crate) fn update(app: &mut App, message: AppMsg) -> Task<AppMsg> {
    match message {
        AppMsg::OpenComment {
            article,
            comment_ids,
            parent,
        } => {
            // Opening first set of comments from an article.
            if let Some(item) = article {
                let item_id = item.id;
                let article_content = std::mem::replace(
                    &mut app.content,
                    ContentScreen::Comments(CommentState {
                        article: item,
                        comments: Vec::new(),
                    }),
                );
                if let ContentScreen::Articles(mut state) = article_content {
                    state.visited.insert(item_id);
                    app.article_state = Some(state);
                }
            }

            let client = app.client.clone();
            Task::batch([
                Task::done(FooterMsg::Fetching).map(AppMsg::Footer),
                Task::perform(
                    async move { client.items(&comment_ids).await },
                    move |result| match result {
                        Ok(comments) => {
                            AppMsg::Comments(CommentMsg::ReceiveComments(comments, parent.clone()))
                        }
                        Err(err) => AppMsg::Footer(FooterMsg::Error(err.to_string())),
                    },
                ),
            ])
        }
        AppMsg::RestoreArticles => match app.article_state.take() {
            Some(state) => {
                app.content = ContentScreen::Articles(state);
                Task::none()
            }
            None => Task::done(ArticleMsg::Fetch {
                limit: 75,
                article_type: ArticleType::Top,
            })
            .map(AppMsg::Articles),
        },
        AppMsg::OpenLink { url, item_id } => {
            open::with(url, "firefox")
                .inspect_err(|err| {
                    error!("Failed to open url {err}");
                })
                .unwrap_or_default();
            Task::done(ArticleMsg::Visited(item_id)).map(AppMsg::Articles)
        }
        AppMsg::ChangeTheme(theme) => {
            app.theme = theme;
            Task::none()
        }
        AppMsg::WindowClose => {
            println!("Window close event");
            Task::none()
        }
        AppMsg::IncreaseScale => {
            app.scale += 0.1;
            Task::done(FooterMsg::Scale(app.scale)).map(AppMsg::Footer)
        }
        AppMsg::DecreaseScale => {
            let new_scale = app.scale - 0.1;
            let int = new_scale * 100.0;

            if int > 10.0 {
                app.scale = new_scale;
            }
            Task::done(FooterMsg::Scale(app.scale)).map(AppMsg::Footer)
        }
        AppMsg::ResetScale => {
            app.scale = 1.0;
            Task::done(FooterMsg::Scale(app.scale)).map(AppMsg::Footer)
        }
        AppMsg::Articles(msg) => match &mut app.content {
            ContentScreen::Articles(state) => state.update(msg),
            ContentScreen::Comments(_) => Task::none(),
        },
        AppMsg::Footer(msg) => app.footer.update(msg),
        AppMsg::Comments(msg) => match &mut app.content {
            ContentScreen::Articles(_) => Task::none(),
            ContentScreen::Comments(state) => state.update(msg),
        },
        AppMsg::Header(msg) => app.header.update(msg),
    }
}

pub(crate) fn view(app: &App) -> iced::Element<AppMsg> {
    let content = match &app.content {
        ContentScreen::Comments(c) => Element::from(column![c.view(), app.footer.view(&app.theme)]),
        ContentScreen::Articles(c) => {
            let col = column![
                app.header.view().map(AppMsg::Header),
                c.view(&app.theme),
                app.footer.view(&app.theme)
            ];
            Element::from(col.spacing(10.))
        }
    };

    container(content).into()
}
