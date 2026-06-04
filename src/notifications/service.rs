use std::sync::Arc;

use futures::StreamExt;
use relm4::Sender;
use wayle_notification::NotificationService;

use super::model::{NotificationToast, newest_first};
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

    let mut popup_updates = service.popups.watch().fuse();

    while popup_updates.next().await.is_some() {
        send_popup_snapshot(&sender, service.as_ref());
    }

    tracing::info!("Notification popup watcher stopped");
    let _ = sender.send(ShellMsg::NotificationsChanged(Vec::new()));
}

fn send_popup_snapshot(sender: &Sender<ShellMsg>, service: &NotificationService) {
    let toasts = service
        .popups
        .get()
        .iter()
        .map(|notification| NotificationToast::from_notification(notification.as_ref()))
        .collect();

    let toasts = newest_first(toasts);

    if sender.send(ShellMsg::NotificationsChanged(toasts)).is_err() {
        tracing::error!("Failed to send notification popup snapshot");
    }
}
