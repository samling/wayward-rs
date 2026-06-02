use crate::bar::state::{BarItemState, BatteryState};
use crate::bar::widget::{BarContext, BarWidget, BarWidgetRuntime, WidgetInstance};
use crate::bar::{BarMsg, style};
use crate::shell::ShellMsg;
use futures::{StreamExt, select};
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use std::sync::Arc;
use wayle_battery::BatteryService;
use wayle_battery::types::DeviceState;

struct BatteryRuntime {
    root: gtk::Label,
}

impl BarWidgetRuntime for BatteryRuntime {
    fn root(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, _context: &BarContext) {
        let text = match state {
            BarItemState::Battery(BatteryState::Ready(text)) => text.as_str(),
            BarItemState::Battery(BatteryState::Unavailable) => "NaN",
            _ => return,
        };

        self.root.set_text(text);
    }
}

pub(crate) struct BatteryWidget;

impl BarWidget for BatteryWidget {
    fn id(&self) -> &'static str {
        "battery"
    }

    fn build(
        &self,
        _instance: &WidgetInstance,
        _sender: &relm4::Sender<BarMsg>,
    ) -> Box<dyn BarWidgetRuntime> {
        let label = gtk::Label::new(Some(&initial_text()));
        style::add_bar_item_classes(&label, "battery");

        Box::new(BatteryRuntime { root: label })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Battery(BatteryState::Unavailable))
    }

    fn start(
        &self,
        sender: Sender<ShellMsg>,
        services: &crate::services::ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        Some(start(sender, services.battery.clone()))
    }
}

pub(super) fn initial_text() -> String {
    "NaN".to_string()
}

pub(crate) fn start(
    sender: Sender<ShellMsg>,
    service: Option<Arc<BatteryService>>,
) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_battery_watcher(sender, service).await;
    })
}

fn battery_text(percentage: f64, state: DeviceState) -> String {
    format!("{percentage:.0}% {state}")
}

async fn run_battery_watcher(sender: Sender<ShellMsg>, service: Option<Arc<BatteryService>>) {
    let Some(service) = service else {
        let _ = sender.send(battery_message(BatteryState::Unavailable));
        return;
    };

    send_battery_snapshot(&sender, service.as_ref());

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

fn battery_message(state: BatteryState) -> ShellMsg {
    ShellMsg::ItemStateChanged(BarItemState::Battery(state))
}
