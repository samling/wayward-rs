use std::process::Command;
use std::sync::Arc;

use futures::{StreamExt, select};
use relm4::Sender;
use relm4::tokio::process::Command as TokioCommand;
use wayle_brightness::{BrightnessService, Percentage};

use crate::bar::state::{BarItemState, BrightnessSnapshot, BrightnessState};
use crate::bar::widget::{BrightnessAction, WidgetAction, WidgetEvent};
use crate::shell::ShellMsg;

pub(super) fn start(
    sender: Sender<ShellMsg>,
    service: Option<Arc<BrightnessService>>,
) -> Option<relm4::JoinHandle<()>> {
    let Some(service) = service else {
        tracing::info!("Brightness widget disabled because brightness service is unavailable");
        return None;
    };

    Some(relm4::spawn(async move {
        run_brightness_watcher(sender, service).await;
    }))
}

pub(super) fn handle_event(event: WidgetEvent, service: Option<Arc<BrightnessService>>) {
    match event.action {
        WidgetAction::Brightness(BrightnessAction::SetBrightness { percent }) => {
            let Some(service) = service else {
                tracing::warn!(
                    "Ignoring brightness action because brightness service is unavailable"
                );
                return;
            };

            relm4::spawn(async move {
                set_brightness(service, percent).await;
            });
        }
        WidgetAction::Brightness(BrightnessAction::RunBlueLightCommand { command }) => {
            run_blue_light_command(&command);
        }
        _ => {}
    }
}

async fn run_brightness_watcher(sender: Sender<ShellMsg>, service: Arc<BrightnessService>) {
    let mut primary_updates = service.primary.watch().fuse();

    let mut device = match service.primary.get() {
        Some(device) => device,
        None => {
            send_unavailable(&sender, "No brightness device");

            loop {
                let Some(maybe_device) = primary_updates.next().await else {
                    return;
                };

                if let Some(device) = maybe_device {
                    break device;
                }
            }
        }
    };

    send_snapshot(&sender, device.as_ref());

    let mut brightness_updates = device.brightness.watch().fuse();

    loop {
        select! {
            update = primary_updates.next() => {
                let Some(maybe_device) = update else {
                    break;
                };

                let Some(updated_device) = maybe_device else {
                    send_unavailable(&sender, "No brightness device");
                    continue;
                };

                device = updated_device;
                brightness_updates = device.brightness.watch().fuse();
                send_snapshot(&sender, device.as_ref());
            }
            update = brightness_updates.next() => {
                if update.is_none() {
                    break;
                }

                send_snapshot(&sender, device.as_ref());
            }
        }
    }

    tracing::info!("Brightness widget watcher stopped");
}

fn send_snapshot(sender: &Sender<ShellMsg>, device: &wayle_brightness::BacklightDevice) {
    let snapshot = BrightnessSnapshot {
        percent: device.percentage().value(),
    };

    let _ = sender.send(ShellMsg::ItemStateChanged(BarItemState::Brightness(
        BrightnessState::Ready(snapshot),
    )));
}

fn send_unavailable(sender: &Sender<ShellMsg>, error: &str) {
    let _ = sender.send(ShellMsg::ItemStateChanged(BarItemState::Brightness(
        BrightnessState::Unavailable(error.to_string()),
    )));
}

async fn set_brightness(service: Arc<BrightnessService>, percent: f64) {
    let Some(device) = service.primary.get() else {
        return;
    };

    if let Err(error) = device.set_percentage(Percentage::new(percent)).await {
        tracing::error!("Failed to set brightness: {error}");
    }
}

fn run_blue_light_command(command: &str) {
    let command = command.trim();

    if command.is_empty() {
        tracing::warn!("No blue-light command configured");
        return;
    }

    if let Err(error) = Command::new("sh").arg("-c").arg(command).spawn() {
        tracing::error!("Failed to run blue-light command: {error}");
    }
}

pub(super) async fn blue_light_enabled(command: &str) -> Option<bool> {
    let command = command.trim();

    if command.is_empty() {
        return None;
    }

    match TokioCommand::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .await
    {
        Ok(status) => Some(status.success()),
        Err(error) => {
            tracing::error!("Failed to run blue-light state command: {error}");
            None
        }
    }
}
