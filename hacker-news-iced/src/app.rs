use crate::{
    articles::{self, ArticleMsg, ArticleState},
    comments::{self, CommentMsg, CommentState},
    config::{save_config, Config},
    footer::{self, FooterMsg, FooterState},
    header::{self, HeaderState},
};
use hacker_news_api::{ApiClient, Item};
use iced::{
    widget::{self, container, pane_grid, Column},
    Size, Task, Theme,
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
    /// Article state.
    pub article_state: ArticleState,
    /// Comment state.
    pub comment_state: Option<CommentState>,
    /// Footer
    pub footer: FooterState,
    /// API Client.
    pub client: Arc<ApiClient>,
    /// Window size
    pub size: Size,
    // Pane grid
    pub panes: pane_grid::State<PaneState>,
}

#[derive(Debug, Copy, Clone)]
pub enum PaneState {
    Articles,
    Comments,
}

#[derive(Debug, Copy, Clone)]
pub enum ScrollBy {
    PageUp,
    PageDown,
    LineUp,
    LineDown,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub enum AppMsg {
    Header(header::HeaderMsg),
    Articles(articles::ArticleMsg),
    Footer(footer::FooterMsg),
    Comments(comments::CommentMsg),
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
    WindowResize(Size),
    ScrollBy(ScrollBy),
    OpenSearch,
    CloseSearch,
    PaneResized(pane_grid::ResizeEvent),
    CommentsClosed,
}

pub fn update(app: &mut App, message: AppMsg) -> Task<AppMsg> {
    match message {
        AppMsg::OpenComment {
            article,
            comment_ids,
            parent,
        } => {
            // Opening first set of comments from an article.
            if let Some(item) = article {
                let item_id = item.id;

                app.comment_state = Some(CommentState {
                    article: item,
                    comments: Vec::new(),
                    search: None,
                });

                app.article_state.visited.insert(item_id);
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
        AppMsg::CommentsClosed => {
            app.comment_state = None;
            Task::none()
        }
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
            save_task(app)
        }
        AppMsg::WindowClose => {
            println!("Window close event");
            Task::none()
        }
        AppMsg::IncreaseScale => {
            app.scale += 0.1;
            Task::batch([
                Task::done(FooterMsg::Scale(app.scale)).map(AppMsg::Footer),
                save_task(app),
            ])
        }
        AppMsg::DecreaseScale => {
            let new_scale = app.scale - 0.1;
            let int = new_scale * 100.0;

            if int > 10.0 {
                app.scale = new_scale;
            }
            Task::batch([
                Task::done(FooterMsg::Scale(app.scale)).map(AppMsg::Footer),
                save_task(app),
            ])
        }
        AppMsg::ResetScale => {
            app.scale = 1.0;
            Task::batch([
                Task::done(FooterMsg::Scale(app.scale)).map(AppMsg::Footer),
                save_task(app),
            ])
        }
        AppMsg::Articles(msg) => app.article_state.update(msg),
        AppMsg::Comments(msg) => app
            .comment_state
            .as_mut()
            .map(|s| s.update(msg))
            .unwrap_or_else(Task::none),
        AppMsg::Footer(msg) => app.footer.update(msg),
        AppMsg::Header(msg) => app.header.update(msg),
        AppMsg::WindowResize(size) => {
            app.size = size;
            save_task(&*app)
        }
        AppMsg::ScrollBy(scroll_by) => {
            // let scroll_id =
            //     scrollable::Id::new(if matches!(app.content, ContentScreen::Articles(_)) {
            //         "articles"
            //     } else {
            //         "comments"
            //     });
            // match scroll_by {
            //     ScrollBy::PageUp => {
            //         scrollable::scroll_by(scroll_id, AbsoluteOffset { x: 0., y: -100. })
            //     }
            //     ScrollBy::PageDown => {
            //         scrollable::scroll_by(scroll_id, AbsoluteOffset { x: 0., y: 100. })
            //     }
            //     ScrollBy::LineUp => {
            //         scrollable::scroll_by(scroll_id, AbsoluteOffset { x: 0., y: -10. })
            //     }
            //     ScrollBy::LineDown => {
            //         scrollable::scroll_by(scroll_id, AbsoluteOffset { x: 0., y: 10. })
            //     }
            //     ScrollBy::Top => scrollable::scroll_to(scroll_id, AbsoluteOffset { x: 0., y: 0. }),
            //     ScrollBy::Bottom => {
            //         scrollable::scroll_to(scroll_id, AbsoluteOffset { x: 0., y: f32::MAX })
            //     }
            // }
            Task::none()
        }
        AppMsg::OpenSearch => {
            Task::done(AppMsg::Header(header::HeaderMsg::OpenSearch))
            // if matches!(app.content, ContentScreen::Articles(_)) {
            //     Task::done(AppMsg::Header(header::HeaderMsg::OpenSearch))
            // } else {
            //     Task::done(AppMsg::Comments(CommentMsg::OpenSearch))
            // }
        }
        AppMsg::CloseSearch => {
            // if matches!(app.content, ContentScreen::Articles(_)) {
            Task::done(AppMsg::Header(header::HeaderMsg::CloseSearch))
            // } else {
            //     Task::done(AppMsg::Comments(CommentMsg::CloseSearch))
            // }
        }
        AppMsg::PaneResized(p) => {
            app.panes.resize(p.split, p.ratio);
            Task::none()
        }
    }
}

pub fn view(app: &App) -> iced::Element<AppMsg> {
    let body = widget::pane_grid(&app.panes, |_pane, state, _is_maximized| {
        pane_grid::Content::new(match state {
            PaneState::Articles => app.article_state.view(&app.theme),
            PaneState::Comments => app
                .comment_state
                .as_ref()
                .map(|s| s.view())
                .unwrap_or_else(|| widget::text("").into()),
        })
    })
    .on_resize(10, AppMsg::PaneResized);

    let main_layout = Column::new()
        .push(app.header.view().map(AppMsg::Header))
        .push(body)
        .push(app.footer.view(&app.theme));

    container(main_layout).into()
}

impl From<&App> for Config {
    fn from(state: &App) -> Self {
        let visited = state.article_state.visited.clone();

        Config {
            scale: state.scale,
            article_count: state.header.article_count,
            article_type: state.header.article_type,
            visited: visited.clone(),
            theme: state.theme.to_string(),
            window_size: (state.size.width, state.size.height),
        }
    }
}

pub fn save_task(app: &App) -> Task<AppMsg> {
    let config = Config::from(app);
    Task::perform(save_config(config), |result| {
        AppMsg::Footer(match result {
            Ok(_) => FooterMsg::Error("Saved".into()),
            Err(err) => FooterMsg::Error(err.to_string()),
        })
    })
}
