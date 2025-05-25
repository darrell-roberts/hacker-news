//! Linux specific settings.
use crate::app::AppMsg;
use iced::futures::Stream;
use log::{error, info};

/// Listen to dconf font scale changes.
pub fn listen_font_scale() -> impl Stream<Item = AppMsg> {
    use iced::futures::SinkExt as _;
    use tokio::{
        io::{AsyncBufReadExt as _, BufReader},
        process::Command,
    };

    iced::stream::channel(100, |mut sender| async move {
        let mut dconf = Command::new("dconf")
            .args(["watch", "/org/gnome/desktop/interface/text-scaling-factor"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = dconf.stdout.take().expect("No dconf stdout");
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await.unwrap() {
            if line != "/org/gnome/desktop/interface/text-scaling-factor" && !line.is_empty() {
                if let Ok(scale) = line.trim().parse::<f64>() {
                    info!("System font scale changed to: {scale}");
                    sender.send(AppMsg::SystemFontScale(scale)).await.unwrap();
                }
            }
        }
    })
}

/// Read the initial dconf font scale
pub fn initial_font_scale() -> Option<f64> {
    use std::process::Command;
    let out = Command::new("dconf")
        .arg("read")
        .arg("/org/gnome/desktop/interface/text-scaling-factor")
        .output()
        .inspect_err(|err| {
            error!("Failed to run dconf: {err}");
        })
        .ok()?;

    String::from_utf8(out.stdout)
        .inspect_err(|err| {
            error!("Failed to parse stdout from dconf: {err}");
        })
        .ok()?
        .split('\n')
        .find_map(|line| line.trim().parse::<f64>().ok())
}
