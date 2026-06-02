use std::sync::Arc;

use futures::StreamExt;
use relm4::Sender;
use wayle_brightness::BrightnessService;

use crate::osd::OsdEvent;
use crate::shell::ShellMsg;

pub(crate) fn start(
    sender: Sender<ShellMsg>,
    service: Option<Arc<BrightnessService>>,
) -> Option<relm4::JoinHandle<()>> {
    let Some(service) = service else {
        tracing::info!("Brightness OSD disabled because brightness service is unavailable");
        return None;
    };
    Some(relm4::spawn(async move {
        run(sender, service).await;
    }))
}

async fn run(sender: Sender<ShellMsg>, service: Arc<BrightnessService>) {
    let mut primary_updates = service.primary.watch().fuse();

    let mut device = match service.primary.get() {
        Some(device) => device,
        None => loop {
            let Some(maybe_device) = primary_updates.next().await else {
                tracing::info!("Brightness primary watcher stopped before a device appeared");
                return;
            };

            if let Some(device) = maybe_device {
                break device;
            }
        },
    };

    send_device_snapshot(&sender, device.as_ref());

    let mut brightness_updates = device.brightness.watch().fuse();

    loop {
        futures::select! {
            update = primary_updates.next() => {
                let Some(maybe_device) = update else {
                    break;
                };

                let Some(updated_device) = maybe_device else {
                    tracing::info!("Primary brightness device disappeared");
                    continue;
                };

                device = updated_device;
                brightness_updates = device.brightness.watch().fuse();

                send_device_snapshot(&sender, device.as_ref());
            }

            update = brightness_updates.next() => {
                if update.is_none() {
                    break;
                }

                send_device_snapshot(&sender, device.as_ref());
            }
        }
    }

    tracing::info!("Brightness watcher stopped");
}

fn send_device_snapshot(sender: &Sender<ShellMsg>, device: &wayle_brightness::BacklightDevice) {
    let percent = device.percentage().value();

    tracing::info!(
        device = %device.name,
        percent,
        "Sending brightness OSD event"
    );

    if sender
        .send(ShellMsg::OsdChanged(OsdEvent::Brightness { percent }))
        .is_err()
    {
        tracing::error!("Failed to send brightness OSD event");
    }
}
