use crate::app::AppModel;
use hacker_news_config::{IndexConfig, index_config, init_logger, search_context};
use hacker_news_search::SearchContext;
use relm4::RelmApp;
use std::sync::{Arc, RwLock};

mod app;

pub struct Config {
    index_config: IndexConfig,
    search_context: Arc<RwLock<SearchContext>>,
}

fn main() -> anyhow::Result<()> {
    let relm = RelmApp::new("dr.github.com");
    let search_context = search_context()?;
    relm4::set_global_css(include_str!("style.css"));

    init_logger("hacker-news-relm4")?;
    let index_config = index_config()?;

    relm.run::<AppModel>(Config {
        index_config,
        search_context,
    });
    Ok(())
}
