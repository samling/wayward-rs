use std::sync::Arc;

use futures::StreamExt;
use relm4::Sender;
use wayle_audio::AudioService;
use wayle_audio::core::device::output::OutputDevice;

use crate::osd::OsdEvent;
use crate::shell::ShellMsg;

pub(crate) fn start(
    sender: Sender<ShellMsg>,
    service: Option<Arc<AudioService>>,
) -> Option<relm4::JoinHandle<()>> {
    let Some(service) = service else {
        tracing::info!("Audio OSD disabled because audio service is unavailable");
        return None;
    };

    Some(relm4::spawn(async move {
        run(sender, service).await;
    }))
}

async fn run(sender: Sender<ShellMsg>, service: Arc<AudioService>) {
    tracing::info!("Audio OSD watcher started");

    let mut default_output_updates = service.default_output.watch().fuse();

    let mut device = match service.default_output.get() {
        Some(device) => device,
        None => loop {
            let Some(maybe_device) = default_output_updates.next().await else {
                tracing::info!("Audio default output watcher stopped before a device appeared");
                return;
            };

            if let Some(device) = maybe_device {
                break device;
            }
        },
    };

    send_device_snapshot(&sender, device.as_ref());

    let mut volume_updates = device.volume.watch().fuse();
    let mut mute_updates = device.muted.watch().fuse();

    loop {
        futures::select! {
            update = default_output_updates.next() => {
                let Some(maybe_device) = update else {
                    break;
                };

                let Some(updated_device) = maybe_device else {
                    tracing::info!("Default audio output disappeared");
                    continue;
                };

                device = updated_device;
                volume_updates = device.volume.watch().fuse();
                mute_updates = device.muted.watch().fuse();

                send_device_snapshot(&sender, device.as_ref());
            }

            update = volume_updates.next() => {
                if update.is_none() {
                    break;
                }

                send_device_snapshot(&sender, device.as_ref());
            }

            update = mute_updates.next() => {
                if update.is_none() {
                    break;
                }

                send_device_snapshot(&sender, device.as_ref());
            }
        }
    }

    tracing::info!("Audio watcher stopped");
}

fn send_device_snapshot(sender: &Sender<ShellMsg>, device: &OutputDevice) {
    let percent = device.volume.get().average_percentage();
    let muted = device.muted.get();

    tracing::info!(percent, muted, "Sending audio OSD event");

    if sender
        .send(ShellMsg::OsdChanged(OsdEvent::Volume { percent, muted }))
        .is_err()
    {
        tracing::error!("Failed to send audio OSD event");
    }
}
