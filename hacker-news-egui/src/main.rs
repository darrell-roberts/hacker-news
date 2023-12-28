use anyhow::{Context, Result};
use app::HackerNewsApp;
use eframe::{icon_data::from_png_bytes, Theme};
use egui::ViewportBuilder;
use event::{ApiEvent, ApiEventHandler, Event, EventHandler};
use hacker_news_api::ResultExt;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc::{self, UnboundedReceiver};

pub mod app;
pub mod event;
pub mod renderer;

pub static SHUT_DOWN: AtomicBool = AtomicBool::new(false);

fn main() -> Result<()> {
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

    let (api_sender, api_receiver) = mpsc::unbounded_channel::<ApiEvent>();
    let (client_sender, client_receiver) = mpsc::unbounded_channel::<Event>();

    let event_handler = EventHandler::new(api_sender.clone(), client_receiver);
    let api_event_handler = ApiEventHandler::new(client, client_sender.clone());

    start_background(api_receiver, api_event_handler)?;

    eframe::run_native(
        "Hacker News",
        native_options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let app = HackerNewsApp::new(cc, event_handler, client_sender);
            let last_request = app.last_request();
            api_sender
                .send(last_request(app.showing))
                .context("Intitial request")
                .log_error_consume();

            Box::new(app)
        }),
    )
    .map_err(|e| anyhow::Error::msg(format!("failed to launch: {e}")))?;

    Ok(())
}

fn start_background(
    mut receiver: UnboundedReceiver<ApiEvent>,
    api_event_handler: ApiEventHandler,
) -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_time()
        .enable_io()
        .build()?;

    std::thread::spawn(move || {
        rt.block_on(async move {
            while !(SHUT_DOWN.load(Ordering::Acquire)) {
                if let Some(event) = receiver.recv().await {
                    api_event_handler.handle_event(event).await;
                }
            }
        });
    });
    Ok(())
}
