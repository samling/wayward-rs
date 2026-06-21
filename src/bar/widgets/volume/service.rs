use std::sync::Arc;

use futures::stream::{self, BoxStream};
use futures::{StreamExt, select};
use relm4::Sender;
use wayle_audio::AudioService;
use wayle_audio::core::device::input::InputDevice;
use wayle_audio::core::device::output::OutputDevice;
use wayle_audio::volume::types::Volume;

use crate::bar::state::{BarItemState, VolumeState};
use crate::bar::widget::{VolumeAction, WidgetAction, WidgetEvent};
use crate::shell::ShellMsg;

use super::model::{AudioDeviceSummary, VolumeSnapshot};

pub(super) fn start(
    sender: Sender<ShellMsg>,
    service: Option<Arc<AudioService>>,
) -> Option<relm4::JoinHandle<()>> {
    let Some(service) = service else {
        tracing::info!("Volume widget disabled because audio service is unavailable");
        return None;
    };

    Some(relm4::spawn(async move {
        run_volume_watcher(sender, service).await;
    }))
}

pub(super) fn handle_event(event: WidgetEvent, service: Option<Arc<AudioService>>) {
    let Some(service) = service else {
        tracing::warn!("Ignoring volume action because audio service is unavailable");
        return;
    };

    match event.action {
        WidgetAction::Volume(VolumeAction::SetOutputVolume { percent }) => {
            relm4::spawn(async move {
                set_output_volume(service, percent).await;
            });
        }
        WidgetAction::Volume(VolumeAction::ToggleOutputMute) => {
            relm4::spawn(async move {
                toggle_output_mute(service).await;
            });
        }
        WidgetAction::Volume(VolumeAction::SetDefaultOutput { key }) => {
            relm4::spawn(async move {
                set_default_output(service, key).await;
            });
        }
        WidgetAction::Volume(VolumeAction::SetDefaultInput { key }) => {
            relm4::spawn(async move {
                set_default_input(service, key).await;
            });
        }
        _ => {}
    }
}

async fn run_volume_watcher(sender: Sender<ShellMsg>, service: Arc<AudioService>) {
    send_snapshot(&sender, &service);

    let mut output_devices = service.output_devices.watch().fuse();
    let mut input_devices = service.input_devices.watch().fuse();
    let mut default_output = service.default_output.watch().fuse();
    let mut default_input = service.default_input.watch().fuse();
    let mut volume = output_volume_stream(service.default_output.get()).fuse();
    let mut mute = output_mute_stream(service.default_output.get()).fuse();

    loop {
        select! {
            update = output_devices.next() => {
                if update.is_none() {
                    break;
                }
                send_snapshot(&sender, &service);
            }
            update = input_devices.next() => {
                if update.is_none() {
                    break;
                }
                send_snapshot(&sender, &service);
            }
            update = default_input.next() => {
                if update.is_none() {
                    break;
                }
                send_snapshot(&sender, &service);
            }
            update = default_output.next() => {
                let Some(device) = update else {
                    break;
                };
                volume = output_volume_stream(device.clone()).fuse();
                mute = output_mute_stream(device).fuse();
                send_snapshot(&sender, &service);
            }
            update = volume.next() => {
                if update.is_none() {
                    volume = output_volume_stream(service.default_output.get()).fuse();
                }
                send_snapshot(&sender, &service);
            }
            update = mute.next() => {
                if update.is_none() {
                    mute = output_mute_stream(service.default_output.get()).fuse();
                }
                send_snapshot(&sender, &service);
            }
        }
    }

    tracing::info!("Volume widget watcher stopped");
}

fn output_volume_stream(device: Option<Arc<OutputDevice>>) -> BoxStream<'static, ()> {
    match device {
        Some(device) => device.volume.watch().map(|_| ()).boxed(),
        None => stream::pending().boxed(),
    }
}

fn output_mute_stream(device: Option<Arc<OutputDevice>>) -> BoxStream<'static, ()> {
    match device {
        Some(device) => device.muted.watch().map(|_| ()).boxed(),
        None => stream::pending().boxed(),
    }
}

fn send_snapshot(sender: &Sender<ShellMsg>, service: &AudioService) {
    let state = match snapshot(service) {
        Ok(snapshot) => VolumeState::Ready(snapshot),
        Err(error) => VolumeState::Unavailable(error),
    };

    let _ = sender.send(ShellMsg::ItemStateChanged(BarItemState::Volume(state)));
}

fn snapshot(service: &AudioService) -> Result<VolumeSnapshot, String> {
    let Some(default_output) = service.default_output.get() else {
        return Err("No default audio output".to_string());
    };

    let default_input = service.default_input.get();

    Ok(VolumeSnapshot {
        percent: default_output.volume.get().average_percentage(),
        muted: default_output.muted.get(),
        outputs: service
            .output_devices
            .get()
            .iter()
            .map(output_summary)
            .collect(),
        inputs: service
            .input_devices
            .get()
            .iter()
            .filter(|device| !device.is_monitor.get())
            .map(input_summary)
            .collect(),
        default_output: Some(default_output.key.index),
        default_input: default_input.map(|device| device.key.index),
    })
}

fn output_summary(device: &Arc<OutputDevice>) -> AudioDeviceSummary {
    AudioDeviceSummary {
        key: device.key.index,
        label: device_label(&device.description.get(), &device.name.get()),
    }
}

fn input_summary(device: &Arc<InputDevice>) -> AudioDeviceSummary {
    AudioDeviceSummary {
        key: device.key.index,
        label: device_label(&device.description.get(), &device.name.get()),
    }
}

fn device_label(description: &str, name: &str) -> String {
    if description.trim().is_empty() {
        name.to_string()
    } else {
        description.to_string()
    }
}

async fn set_output_volume(service: Arc<AudioService>, percent: f64) {
    let Some(device) = service.default_output.get() else {
        return;
    };

    let channels = device.volume.get().channels().max(1);

    if let Err(error) = device
        .set_volume(Volume::from_percentage(percent.clamp(0.0, 100.0), channels))
        .await
    {
        tracing::error!("Failed to set output volume: {error}");
    }
}

async fn toggle_output_mute(service: Arc<AudioService>) {
    let Some(device) = service.default_output.get() else {
        return;
    };

    let muted = !device.muted.get();

    if let Err(error) = device.set_mute(muted).await {
        tracing::error!("Failed to toggle output mute: {error}");
    }
}

async fn set_default_output(service: Arc<AudioService>, key: u32) {
    let Some(device) = service
        .output_devices
        .get()
        .into_iter()
        .find(|device| device.key.index == key)
    else {
        return;
    };

    if let Err(error) = device.set_as_default().await {
        tracing::error!("Failed to set default output: {error}");
    }
}

async fn set_default_input(service: Arc<AudioService>, key: u32) {
    let Some(device) = service
        .input_devices
        .get()
        .into_iter()
        .find(|device| device.key.index == key)
    else {
        return;
    };

    if let Err(error) = device.set_as_default().await {
        tracing::error!("Failed to set default input: {error}");
    }
}
