use anyhow::Context;
use app::HackerNewsApp;
use eframe::{icon_data::from_png_bytes, Theme};
use egui::ViewportBuilder;
use event::{ClientEvent, ClientEventHandler, Event, EventHandler};
use hacker_news_api::ResultExt;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc;

pub mod app;
pub mod event;
pub mod renderer;

pub static SHUT_DOWN: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let client =
        Arc::new(hacker_news_api::ApiClient::new().context("Could not create api client")?);

    let icon = from_png_bytes(include_bytes!("../assets/icon.png"))?;

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_icon(icon),
        persist_window: true,
        // For now only light theme.
        follow_system_theme: false,
        default_theme: Theme::Light,
        ..Default::default()
    };

    eframe::run_native(
        "Hacker News",
        native_options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let ctx = cc.egui_ctx.clone();
            let (sender, mut receiver) = mpsc::unbounded_channel::<ClientEvent>();
            let (local_sender, client_receiver) = mpsc::unbounded_channel::<Event>();

            let event_handler = EventHandler::new(sender.clone(), client_receiver);
            let client_event_handler =
                ClientEventHandler::new(client.clone(), ctx, local_sender.clone());

            let _handle = tokio::spawn(async move {
                while !(SHUT_DOWN.load(Ordering::Acquire)) {
                    if let Some(event) = receiver.recv().await {
                        client_event_handler.handle_event(event).await;
                    }
                }
            });

            let app = HackerNewsApp::new(cc, event_handler, local_sender);
            let last_request = app.last_request();
            sender.send(last_request(app.showing)).log_error_consume();

            Box::new(app)
        }),
    )
    .map_err(|e| anyhow::Error::msg(format!("failed to launch: {e}")))?;
    Ok(())
}
