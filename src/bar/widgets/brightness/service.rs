use std::process::Command;
use std::sync::Arc;

use futures::{StreamExt, select};
use relm4::Sender;
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
        WidgetAction::Brightness(BrightnessAction::SetSunsetrPreset { preset }) => {
            set_sunsetr_preset(&preset);
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

fn set_sunsetr_preset(preset: &str) {
    let preset = preset.trim();

    if preset.is_empty() {
        tracing::warn!("No sunsetr preset configured");
        return;
    }

    if let Err(error) = Command::new("sunsetr").arg("preset").arg(preset).spawn() {
        tracing::error!("Failed to set sunsetr preset {preset}: {error}");
    }
}
