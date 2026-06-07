use std::sync::Arc;

use wayle_notification::NotificationService;

use crate::bar::widget::{WidgetAction, WidgetEvent};

pub(super) fn handle_event(event: WidgetEvent, service: Option<Arc<NotificationService>>) {
    match event.action {
        WidgetAction::InvokeNotificationDefault { id } => {
            invoke_default(id, service);
        }
        WidgetAction::InvokeNotificationAction { id, action_id } => {
            invoke_action(id, action_id, service);
        }
        WidgetAction::DismissNotification { id } => {
            dismiss_notification(id, service);
        }
        WidgetAction::DismissAllNotifications => {
            dismiss_all(service);
        }
        _ => {}
    }
}

fn invoke_default(id: u32, service: Option<Arc<NotificationService>>) {
    let Some(service) = service else {
        tracing::info!("Cannot invoke notification because notification service is unavailable");
        return;
    };

    relm4::spawn(async move {
        let Some(notification) = crate::notifications::actions::notification_by_id(service.as_ref(), id) else {
            tracing::info!(id, "Default notification action target disappeared");
            return;
        };

        if let Some(action) = notification.default_action.get() {
            let action_id = action.id;

            if let Err(error) = notification.invoke(&action_id).await {
                tracing::error!(
                    id,
                    action_id = %action_id,
                    "Failed to invoke default notification action: {error}"
                );
            }
        }

        service.dismiss_popup(id);
    });
}


fn invoke_action(id: u32, action_id: String, service: Option<Arc<NotificationService>>) {
    let Some(service) = service else {
        tracing::info!("Cannot invoke notification action because notification service is unavailable");
        return;
    };

    relm4::spawn(async move {
        let Some(notification) = crate::notifications::actions::notification_by_id(service.as_ref(), id) else {
            tracing::info!(id, action_id, "Notification action target disappeared");
            return;
        };

        if let Err(error) = notification.invoke(&action_id).await {
            tracing::error!(id, action_id, "Failed to invoke notification action: {error}");
        }

        service.dismiss_popup(id);
    });
}

fn dismiss_notification(id: u32, service: Option<Arc<NotificationService>>) {
    let Some(service) = service else {
        tracing::info!("Cannot dismiss notification because notification service is unavailable");
        return;
    };

    let Some(notification) = crate::notifications::actions::notification_by_id(service.as_ref(), id) else {
        tracing::info!(id, "Notification target disappeared");
        return;
    };

    notification.dismiss();
    service.dismiss_popup(id);
}

fn dismiss_all(service: Option<Arc<NotificationService>>) {
    let Some(service) = service else {
        tracing::info!("Cannot dismiss notifications because notification service is unavailable");
        return;
    };

    relm4::spawn(async move {
        if let Err(error) = service.dismiss_all().await {
            tracing::error!("Failed to dismiss all notifications: {error}");
        }
    });
}