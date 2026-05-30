use crate::app::AppModel;
use hacker_news_config::search_context;
use relm4::RelmApp;

mod app;

fn main() -> anyhow::Result<()> {
    let relm = RelmApp::new("dr.github.com");
    let search_context = search_context()?;
    relm4::set_global_css(include_str!("style.css"));
    relm.run::<AppModel>(search_context);
    Ok(())
}
