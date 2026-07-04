//! Header component
use hacker_news_api::ArticleType;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    *,
};

pub struct HeaderModel;

#[derive(Debug)]
pub enum HeaderOutMsg {
    ChangeView(ArticleType),
    Refresh,
}

#[relm4::component(pub)]
impl SimpleComponent for HeaderModel {
    type Init = ();
    type Input = ();
    type Output = HeaderOutMsg;

    view! {
        #[root]
        header_bar = gtk::HeaderBar {
            #[wrap(Some)]
            set_title_widget = &gtk::Box {
                gtk::Label {
                    set_label: "Hacker News"
                }
            },
            pack_end = &gtk::MenuButton {
                set_icon_name: "open-menu-symbolic",
                set_menu_model: Some(&main_menu),
            },
        }
    }

    menu! {
        main_menu: {
            "Article Type" {
                "Top" => SelectTopAction,
                "Best" => SelectBestAction,
                "New" => SelectNewAction,
            },
            "Refresh" => RefreshAction,
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let top_action: RelmAction<SelectTopAction> = {
            let sender = sender.clone();
            RelmAction::new_stateless(move |_| {
                if let Err(err) = sender.output(HeaderOutMsg::ChangeView(ArticleType::Top)) {
                    eprint!("Failed to change view: {err:?}");
                }
            })
        };
        let best_action: RelmAction<SelectBestAction> = {
            let sender = sender.clone();
            RelmAction::new_stateless(move |_| {
                if let Err(err) = sender.output(HeaderOutMsg::ChangeView(ArticleType::Best)) {
                    eprint!("Failed to change view: {err:?}");
                }
            })
        };

        let new_action: RelmAction<SelectNewAction> = {
            let sender = sender.clone();
            RelmAction::new_stateless(move |_| {
                if let Err(err) = sender.output(HeaderOutMsg::ChangeView(ArticleType::New)) {
                    eprint!("Failed to change view: {err:?}");
                }
            })
        };

        let refresh_action: RelmAction<RefreshAction> = {
            RelmAction::new_stateless(move |_| {
                if let Err(err) = sender.output(HeaderOutMsg::Refresh) {
                    eprintln!("Failed to refresh: {err:?}");
                }
            })
        };

        let mut group = RelmActionGroup::<WindowActionGroup>::new();
        group.add_action(top_action);
        group.add_action(best_action);
        group.add_action(new_action);
        group.add_action(refresh_action);

        let widgets = view_output!();
        group.register_for_widget(&widgets.header_bar);

        let model = HeaderModel;
        ComponentParts { model, widgets }
    }
}

new_action_group!(WindowActionGroup, "win");
new_stateless_action!(SelectTopAction, WindowActionGroup, "top");
new_stateless_action!(SelectBestAction, WindowActionGroup, "best");
new_stateless_action!(SelectNewAction, WindowActionGroup, "new");
new_stateless_action!(RefreshAction, WindowActionGroup, "refresh");
