use futures::{FutureExt, StreamExt, select};
use gtk::prelude::*;
use relm4::gtk;
use relm4::Sender;
use wayle_battery::BatteryService;
use wayle_battery::types::DeviceState;
use super::BarMsg;

pub(super) fn initial_text() -> String {
    "NaN".to_string()
}

pub(super) fn start(sender: Sender<BarMsg>) {
    relm4::spawn(async move {
        run_battery_watcher(sender).await;
    });
}

fn battery_text(percentage: f64, state: DeviceState) -> String{
    format!("{percentage:.0}% {state}")
}

async fn run_battery_watcher(sender: Sender<BarMsg>) {
    let Ok(service) = BatteryService::new().await else {
        let _ = sender.send(BarMsg::BatteryUnavailable);
        return;
    };

    send_battery_snapshot(&sender, &service);

    let mut percentage_updates = service.device.percentage.watch().fuse();
    let mut state_updates = service.device.state.watch().fuse();

    loop {
        select! {
            update = percentage_updates.next() => {
                if update.is_none() {
                    break;
                }

                send_battery_snapshot(&sender, &service);
            }
            update = state_updates.next() => {
                if update.is_none() {
                    break;
                }
                send_battery_snapshot(&sender, &service);
            }
        }
    }

    send_battery_snapshot(&sender, &service);

    let mut percentage_updates = service.device.percentage.watch();

    while percentage_updates.next().await.is_some() {
        send_battery_snapshot(&sender, &service);
    }
}

fn send_battery_snapshot(sender: &Sender<BarMsg>, service: &BatteryService) {
    let percentage = service.device.percentage.get();
    let state = service.device.state.get();
    let text = battery_text(percentage, state);

    let _ = sender.send(BarMsg::BatteryChanged(text));
}

pub(super) fn render(label: &gtk::Label) {
    label.add_css_class("bar-item");
    label.add_css_class("battery");
}