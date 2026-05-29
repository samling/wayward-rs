use crate::bar::item;
use futures::{StreamExt, select};
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::prelude::BoxExt;
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

fn battery_text(percentage: f64, state: DeviceState) -> String {
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

    let _ = sender.send(BarMsg::BatteryUnavailable);
}

fn send_battery_snapshot(sender: &Sender<BarMsg>, service: &BatteryService) {
    let percentage = service.device.percentage.get();
    let state = service.device.state.get();
    let text = battery_text(percentage, state);

    let _ = sender.send(BarMsg::BatteryChanged(text));
}

pub(super) fn render(container: &gtk::Box, text: &str) {
    let label = gtk::Label::new(Some(text));
    item::style_label(&label, "battery");
    container.append(&label);
}
