use app::App;
use iced::{advanced::graphics::core::window, Application};

mod app;
mod articles;
mod comment;
mod header;
pub mod richtext;
pub mod widget;

fn main() -> iced::Result {
    App::run(iced::Settings {
        window: window::Settings {
            size: iced::Size {
                width: 768.,
                height: 1024.,
            },
            ..Default::default()
        },
        ..Default::default()
    })
}
