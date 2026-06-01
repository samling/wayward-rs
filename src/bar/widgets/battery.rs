use crate::bar::state::{BarItemState, BatteryState};
use crate::bar::widget::{BarWidget, WidgetInstance};
use crate::bar::{Bar, style};
use crate::shell::ShellMsg;
use futures::{StreamExt, select};
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::prelude::BoxExt;
use wayle_battery::BatteryService;
use wayle_battery::types::DeviceState;

pub(crate) struct BatteryWidget;

impl BarWidget for BatteryWidget {
    fn id(&self) -> &'static str {
        "battery"
    }

    fn render(&self, bar: &Bar, _instance: &WidgetInstance, container: &gtk::Box) {
        let fallback = initial_text();

        let text = bar
            .item_states()
            .iter()
            .find_map(|state| match state {
                BarItemState::Battery(BatteryState::Ready(text)) => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or(fallback.as_str());

        render(container, text);
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Battery(BatteryState::Unavailable))
    }

    fn start(&self, sender: Sender<ShellMsg>) -> Option<relm4::JoinHandle<()>> {
        Some(start(sender))
    }
}

pub(super) fn initial_text() -> String {
    "NaN".to_string()
}

pub(crate) fn start(sender: Sender<ShellMsg>) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_battery_watcher(sender).await;
    })
}

fn battery_text(percentage: f64, state: DeviceState) -> String {
    format!("{percentage:.0}% {state}")
}

async fn run_battery_watcher(sender: Sender<ShellMsg>) {
    let Ok(service) = BatteryService::new().await else {
        let _ = sender.send(battery_message(BatteryState::Unavailable));
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

    let _ = sender.send(battery_message(BatteryState::Unavailable));
}

fn send_battery_snapshot(sender: &Sender<ShellMsg>, service: &BatteryService) {
    let percentage = service.device.percentage.get();
    let state = service.device.state.get();
    let text = battery_text(percentage, state);

    let _ = sender.send(battery_message(BatteryState::Ready(text)));
}

pub(super) fn render(container: &gtk::Box, text: &str) {
    let label = gtk::Label::new(Some(text));
    style::style_label(&label, "battery");
    container.append(&label);
}

fn battery_message(state: BatteryState) -> ShellMsg {
    ShellMsg::ItemStateChanged(BarItemState::Battery(state))
}
