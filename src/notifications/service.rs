use std::sync::Arc;

use futures::{StreamExt, select};
use relm4::Sender;
use wayle_notification::NotificationService;

use super::model::{NotificationToast, newest_first};
use crate::bar::state::{BarItemState, NotificationState};
use crate::shell::ShellMsg;

pub(crate) fn start(
    sender: Sender<ShellMsg>,
    service: Option<Arc<NotificationService>>,
) -> Option<relm4::JoinHandle<()>> {
    let Some(service) = service else {
        tracing::info!("Notification toasts disabled because notification service is unavailable");
        return None;
    };

    Some(relm4::spawn(async move {
        run(sender, service).await;
    }))
}

async fn run(sender: Sender<ShellMsg>, service: Arc<NotificationService>) {
    tracing::info!("Notification popup watcher started");

    send_popup_snapshot(&sender, service.as_ref());
    send_active_snapshot(&sender, service.as_ref());

    let mut popup_updates = service.popups.watch().fuse();
    let mut notification_updates = service.notifications.watch().fuse();

    loop {
        select! {
            update = popup_updates.next() => {
                if update.is_none() {
                    break;
                }

                send_popup_snapshot(&sender, service.as_ref());
            }
            update = notification_updates.next() => {
                if update.is_none() {
                    break;
                }

                send_active_snapshot(&sender, service.as_ref());
            }
        }
    }

    tracing::info!("Notification popup watcher stopped");
    let _ = sender.send(ShellMsg::PopupNotificationsChanged(Vec::new()));
    let _ = sender.send(ShellMsg::ItemStateChanged(BarItemState::Notifications(
        NotificationState::Unavailable,
    )));
}

fn send_popup_snapshot(sender: &Sender<ShellMsg>, service: &NotificationService) {
    let toasts = service
        .popups
        .get()
        .iter()
        .map(|notification| NotificationToast::from_notification(notification.as_ref()))
        .collect();

    let toasts = newest_first(toasts);

    if sender
        .send(ShellMsg::PopupNotificationsChanged(toasts))
        .is_err()
    {
        tracing::error!("Failed to send notification popup snapshot");
    }
}

fn send_active_snapshot(sender: &Sender<ShellMsg>, service: &NotificationService) {
    let notifications = service
        .notifications
        .get()
        .iter()
        .map(|notification| NotificationToast::from_notification(notification.as_ref()))
        .collect();

    let notifications = newest_first(notifications);

    if sender
        .send(ShellMsg::ItemStateChanged(BarItemState::Notifications(
            NotificationState::Ready(notifications),
        )))
        .is_err()
    {
        tracing::error!("Failed to send active notification snapshot");
    }
}
