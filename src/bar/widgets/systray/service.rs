use futures::StreamExt;
use relm4::Sender;
use std::sync::{Arc, Mutex};
use wayle_systray::SystemTrayService;
use wayle_systray::types::Coordinates;

use super::model::SystrayItemSummary;
use crate::bar::state::{BarItemState, SystrayState};
use crate::bar::widget::{WidgetAction, WidgetEvent};
use crate::shell::ShellMsg;

static SERVICE: Mutex<Option<Arc<SystemTrayService>>> = Mutex::new(None);

pub fn start(sender: relm4::Sender<ShellMsg>) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_systray_watcher(sender).await;
    })
}

pub async fn run_systray_watcher(sender: Sender<ShellMsg>) {
    let Ok(service) = SystemTrayService::new().await else {
        let _ = sender.send(systray_message(SystrayState::Unavailable));
        return;
    };

    store_service(service.clone());

    send_systray_snapshot(&sender, service.as_ref());

    let mut item_updates = service.items.watch();

    while item_updates.next().await.is_some() {
        send_systray_snapshot(&sender, service.as_ref());
    }

    let _ = sender.send(systray_message(SystrayState::Unavailable));
}

pub(crate) fn handle_event(event: WidgetEvent) {
    match event.action {
        WidgetAction::Clicked {
            item_id,
            button,
            x,
            y,
        } => {
            let bus_name = item_id;
            handle_click(bus_name, button, x, y);
        }
    }
}

fn handle_click(bus_name: String, button: u32, x: i32, y: i32) {
    let Some(service) = current_service() else {
        tracing::warn!("Ignoring systray click before service is ready");
        return;
    };

    relm4::spawn(async move {
        let Some(item) = service
            .items
            .get()
            .into_iter()
            .find(|item| item.bus_name.get() == bus_name)
        else {
            tracing::warn!("Systray item disappeared before click could be handled: {bus_name}");
            return;
        };

        let coords = Coordinates::new(x, y);

        let result = match button {
            1 if item.item_is_menu.get() => item.context_menu(coords).await,
            1 => item.activate(coords).await,
            2 => item.secondary_activate(coords).await,
            3 => item.context_menu(coords).await,
            _ => {
                return;
            }
        };

        if let Err(error) = result {
            tracing::error!("Failed to handle systray click for {bus_name}: {error}");
        }
    });
}

fn send_systray_snapshot(sender: &Sender<ShellMsg>, service: &SystemTrayService) {
    let items = service
        .items
        .get()
        .iter()
        .map(|item| SystrayItemSummary::from_wayle_item(item))
        .collect();

    let _ = sender.send(systray_message(SystrayState::Ready(items)));
}

fn systray_message(state: SystrayState) -> ShellMsg {
    ShellMsg::ItemStateChanged(BarItemState::Systray(state))
}

fn store_service(service: Arc<SystemTrayService>) {
    let Ok(mut stored_service) = SERVICE.lock() else {
        tracing::error!("Failed to lock systray service");
        return;
    };

    *stored_service = Some(service);
}

fn current_service() -> Option<Arc<SystemTrayService>> {
    let Ok(stored_service) = SERVICE.lock() else {
        tracing::error!("Failed to lock systray service");
        return None;
    };

    stored_service.clone()
}

pub(crate) fn item_by_bus_name(
    bus_name: &str,
) -> Option<std::sync::Arc<wayle_systray::core::item::TrayItem>> {
    current_service()?
        .items
        .get()
        .into_iter()
        .find(|item| item.bus_name.get() == bus_name)
}
