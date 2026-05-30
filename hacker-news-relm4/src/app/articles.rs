//! Article model and factory component.
use hacker_news_search::api::Story;
use relm4::{gtk::prelude::*, prelude::FactoryComponent, *};

pub struct ArticleModel {
    article: Story,
    by_string: String,
    comment_count: String,
}

#[derive(Debug)]
pub enum ArticleOutMsg {
    OpenComment(u64),
}

#[relm4::factory(pub)]
impl FactoryComponent for ArticleModel {
    type ParentWidget = gtk::Box;
    type CommandOutput = ();
    type Input = ();
    type Output = ArticleOutMsg;
    type Init = Story;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            add_css_class: "article",
            add_controller = gtk::GestureClick {
                connect_pressed[id = self.article.id] =>  move |_, _n_press, _, _| {
                    if let Err(err) = sender.output(ArticleOutMsg::OpenComment(id)) {
                        eprintln!("Failed to send open comment message: {err:?}");
                    }
                }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,


                #[name(title)]
                gtk::Label {
                    set_wrap: true,
                    set_label: &self.article.title,
                }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,

                gtk::Label {
                    add_css_class: "by_author",
                    set_label: &self.by_string
                },

                if self.article.descendants > 0 {
                    gtk::Label {
                        add_css_class: "by_author",
                        set_label: &self.comment_count
                    }
                } else {
                    gtk::Label {}
                }
            }
        }

    }

    fn init_model(article: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            by_string: format!("by: {}", &article.by),
            comment_count: format!("{} ", &article.descendants),
            article,
        }
    }
}
