use std::sync::Arc;

use wayle_notification::{NotificationService, core::notification::Notification};

pub(crate) fn notification_by_id(
    service: &NotificationService,
    id: u32,
) -> Option<Arc<Notification>> {
    service
        .popups
        .get()
        .into_iter()
        .chain(service.notifications.get())
        .find(|notification| notification.id == id)
}
