//! Main application model and window.
use articles::ArticleModel;
use comments::CommentModel;
use futures::{StreamExt, channel::mpsc};
use hacker_news_api::ArticleType;
use hacker_news_search::{IndexStats, RebuildProgress, SearchContext, rebuild_index};
use header::HeaderModel;
use relm4::{
    gtk::{
        glib::idle_add_local_once,
        prelude::{
            AdjustmentExt, BoxExt as _, ButtonExt as _, GtkWindowExt as _, OrientableExt as _,
            WidgetExt,
        },
    },
    prelude::FactoryVecDeque,
    *,
};
use std::sync::{Arc, RwLock};

mod articles;
mod comments;
mod header;

pub struct AppModel {
    header: Controller<HeaderModel>,
    articles: FactoryVecDeque<ArticleModel>,
    comments: FactoryVecDeque<CommentModel>,
    search_context: Arc<RwLock<SearchContext>>,
    updating: bool,
    total_stories: usize,
    progress_received: usize,
    comment_stack: Vec<u64>,
    comment_page_offset: usize,
    comment_page: u8,
    total_comment_pages: u8,
    active_comment_id: u64,
}

#[derive(Debug)]
pub enum AppMsg {
    Fetch,
    Change(ArticleType),
    OpenComment(u64),
    CloseComment,
    Refresh,
    Progress(RebuildProgress),
    IndexStatus(IndexStats),
    CommentPrev,
    CommentNext,
}

#[relm4::component(pub)]
impl Component for AppModel {
    type Init = Arc<RwLock<SearchContext>>;
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Window {
            set_default_width: 1024,
            set_default_height: 1024,
            set_titlebar: Some(model.header.widget()),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Paned {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_vexpand: true,
                    set_shrink_start_child: true,
                    set_shrink_end_child: true,

                    #[wrap(Some)]
                    #[name(article_scroll)]
                    set_start_child =  &gtk::ScrolledWindow {
                        add_css_class: "article_pane",

                        #[local_ref]
                        articles_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5,
                        }
                    },

                    #[wrap(Some)]
                    #[name(comment_scroll)]
                    set_end_child = &gtk::ScrolledWindow {
                        add_css_class: "comment_pane",

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Box {
                                gtk::Button {
                                    set_icon_name: "go-previous",
                                    connect_clicked[sender] => move |_| {
                                        sender.input_sender().emit(AppMsg::CloseComment);
                                    }
                                },
                            },

                            // Update to use ListView as a tree view.
                            // https://docs.gtk.org/gtk4/class.ListView.html
                            #[local_ref]
                            comments_box -> gtk::Box {
                                set_vexpand: true,
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 5,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_homogeneous: true,

                                    #[name(comment_prev)]
                                    gtk::Button {
                                        set_icon_name: "go-previous",
                                        connect_clicked[sender] => move |_| {
                                            sender.input_sender().emit(AppMsg::CommentPrev);
                                        }
                                    },

                                    #[name(comments_footer)]
                                    gtk::Label {
                                        set_label: "end"
                                    },

                                    #[name(comment_next)]
                                    gtk::Button {
                                        set_icon_name: "go-next",
                                        connect_clicked[sender] => move |_| {
                                            sender.input_sender().emit(AppMsg::CommentNext);
                                        }
                                    },
                                }

                            },
                        }
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    add_css_class: "status_line",

                    #[name(status_label)]
                    gtk::Label {
                        set_hexpand: true,
                        set_label: "",
                        set_halign: gtk::Align::Start
                    },

                    #[name(progress_bar)]
                    gtk::ProgressBar {
                        set_visible: model.updating,
                        set_text: Some("Fetching"),
                        set_show_text: true
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header: Controller<HeaderModel> =
            HeaderModel::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    header::HeaderOutMsg::ChangeView(ty) => AppMsg::Change(ty),
                    header::HeaderOutMsg::Refresh => AppMsg::Refresh,
                });
        let articles = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |msg| match msg {
                articles::ArticleOutMsg::OpenComment(id) => AppMsg::OpenComment(id),
            });

        let comments = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |msg| match msg {
                comments::CommentOutMsg::OpenComment(id) => AppMsg::OpenComment(id),
            });

        let model = AppModel {
            header,
            articles,
            comments,
            search_context: init,
            updating: false,
            total_stories: 0,
            progress_received: 0,
            comment_stack: Vec::new(),
            comment_page_offset: 0,
            active_comment_id: 0,
            comment_page: 0,
            total_comment_pages: 0,
        };

        sender.input(AppMsg::Fetch);

        let articles_box = model.articles.widget();
        let comments_box = model.comments.widget();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppMsg::Fetch => {
                let result = self.search_context.read().unwrap().top_stories(75, 0);
                match result {
                    Ok(top_stories) => {
                        self.articles.guard().clear();
                        self.articles.extend(top_stories);
                    }
                    Err(err) => {
                        eprintln!("Failed to fetch {err}");
                    }
                }
            }
            AppMsg::Change(ty) => {
                widgets.comment_scroll.vadjustment().set_value(0.0);
                widgets.article_scroll.vadjustment().set_value(0.0);
                self.comments.guard().clear();
                self.search_context
                    .write()
                    .unwrap()
                    .activate_index(ty)
                    .unwrap();
                sender.input(AppMsg::Fetch);
            }
            AppMsg::OpenComment(id) => {
                self.active_comment_id = id;
                let comments =
                    self.search_context
                        .read()
                        .unwrap()
                        .comments(id, 10, self.comment_page_offset);
                if self.comment_page_offset == 0 {
                    self.comment_page = 1;
                }

                match comments {
                    Ok((comments, total)) => {
                        self.comment_stack.push(id);
                        self.comments.guard().clear();
                        self.comments.extend(comments);
                        let adjustment = widgets.comment_scroll.vadjustment();
                        idle_add_local_once(move || {
                            adjustment.set_value(0.0);
                        });
                        self.total_comment_pages = {
                            let (mut pages, rem) = (total / 10, total % 10);
                            if rem > 0 {
                                pages += 1;
                            }
                            pages as u8
                        };
                        widgets.comments_footer.set_label(&format!(
                            "{} / {}",
                            self.comment_page, self.total_comment_pages
                        ));
                        widgets.comment_prev.set_sensitive(self.comment_page != 1);
                        widgets
                            .comment_next
                            .set_sensitive(self.comment_page != self.total_comment_pages);
                    }
                    Err(err) => {
                        eprintln!("Failed to fetch comments: {err}");
                    }
                }
            }
            AppMsg::CloseComment => {
                self.comment_stack.pop();
                if let Some(last_comment) = self.comment_stack.pop() {
                    sender
                        .input_sender()
                        .emit(AppMsg::OpenComment(last_comment));
                }
                if self.comment_stack.is_empty() {
                    self.comments.guard().clear();
                }
            }
            AppMsg::CommentNext => {
                self.comment_page_offset += 10;
                self.comment_page += 1;
                sender.input(AppMsg::OpenComment(self.active_comment_id));
            }
            AppMsg::CommentPrev => {
                self.comment_page_offset = self.comment_page_offset.saturating_sub(10);
                self.comment_page = self.comment_page.saturating_sub(1);
                sender.input(AppMsg::OpenComment(self.active_comment_id));
            }
            AppMsg::Refresh => {
                let category = self.search_context.read().unwrap().active_category();
                let (tx, mut rx) = mpsc::channel::<RebuildProgress>(100);
                let fut = rebuild_index(self.search_context.clone(), category, tx);
                let sender_copy = sender.clone();

                sender.oneshot_command(async move {
                    let (_, index_stats) = futures::join!(
                        async {
                            while let Some(progress) = rx.next().await {
                                sender_copy.input(AppMsg::Progress(progress));
                            }
                        },
                        fut
                    );

                    match index_stats {
                        Ok(stats) => {
                            sender_copy.input(AppMsg::IndexStatus(stats));
                        }
                        Err(err) => {
                            eprintln!("Failed to rebuild index: {err}");
                        }
                    }
                });
            }
            AppMsg::IndexStatus(stats) => {
                widgets.status_label.set_text(&format!(
                    "{} docs {} build: {} on {}",
                    stats.category.as_str(),
                    stats.total_stories,
                    stats.build_time.as_secs(),
                    stats.built_on,
                ));
            }
            AppMsg::Progress(progress) => match progress {
                RebuildProgress::Started(total_stories) => {
                    println!("Rebuilding {total_stories} stories");
                    self.updating = true;
                    self.total_stories = total_stories;
                    self.progress_received = 0;
                    widgets.progress_bar.set_visible(true);
                    widgets.progress_bar.set_fraction(0.0);
                }
                RebuildProgress::StoryCompleted => {
                    self.progress_received += 1;
                    let progress = self.progress_received as f64 / self.total_stories as f64;
                    widgets.progress_bar.set_fraction(progress);
                }
                RebuildProgress::Completed => {
                    self.updating = false;
                    self.progress_received = 0;
                    widgets.progress_bar.set_visible(false);
                    widgets.progress_bar.set_fraction(0.0);
                }
            },
        }
    }
}
