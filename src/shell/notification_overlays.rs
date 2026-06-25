use relm4::prelude::ComponentSender;

use super::{Shell, monitors};

pub(super) struct RunningNotificationWindow {
    pub(super) connector: String,
    pub(super) window: crate::notifications::window::NotificationWindow,
}

fn notification_target_connector<'a>(
    configured_connector: Option<&'a str>,
    focused_connector: Option<&'a str>,
    available_connectors: &[&str],
) -> Option<&'a str> {
    if let Some(configured_connector) = configured_connector {
        if available_connectors.contains(&configured_connector) {
            return Some(configured_connector);
        }
    }

    focused_connector
}

impl Shell {
    pub(super) fn reconcile_notification_windows(&mut self, sender: &ComponentSender<Self>) {
        let monitors = monitors::available();

        self.notification_windows.retain(|running| {
            monitors.iter().any(|monitor| {
                monitors::connector(monitor).as_deref() == Some(running.connector.as_str())
            })
        });

        for monitor in monitors {
            let Some(connector) = monitors::connector(&monitor) else {
                continue;
            };

            if self
                .notification_windows
                .iter()
                .any(|running| running.connector == connector)
            {
                continue;
            }

            self.notification_windows.push(RunningNotificationWindow {
                connector,
                window: crate::notifications::window::NotificationWindow::new(
                    &monitor,
                    sender.input_sender().clone(),
                ),
            });
        }

        self.show_notifications();
    }

    pub(super) fn show_notifications(&self) {
        let available_connectors: Vec<_> = self
            .notification_windows
            .iter()
            .map(|running| running.connector.as_str())
            .collect();

        let Some(target_connector) = notification_target_connector(
            self.config.notifications.monitor.as_deref(),
            self.focused_monitor_connector.as_deref(),
            &available_connectors,
        ) else {
            for running in &self.notification_windows {
                running.window.set_toasts(&[]);
            }
            return;
        };

        for running in &self.notification_windows {
            if running.connector == target_connector {
                running.window.set_toasts(&self.popup_notifications);
            } else {
                running.window.set_toasts(&[]);
            }
        }
    }

    pub(super) fn dismiss_notification_popup(&self, id: u32) {
        let Some(service) = self.services.notification.as_ref() else {
            tracing::info!(
                "Cannot dismiss notification popup because notification service is unavailable"
            );
            return;
        };

        service.dismiss_popup(id);
    }
    pub(super) fn invoke_notification_action(&self, id: u32, action_id: String) {
        let Some(service) = self.services.notification.clone() else {
            tracing::info!(
                "Cannot invoke notification action because notification service is unavailable"
            );
            return;
        };

        relm4::spawn(async move {
            if let Some(notification) =
                crate::notifications::actions::notification_by_id(service.as_ref(), id)
            {
                if let Err(error) = notification.invoke(&action_id).await {
                    tracing::error!(
                        id,
                        action_id,
                        "Failed to invoke notification action: {error}"
                    );
                }
            } else {
                tracing::info!(id, action_id, "Notification action target disappeared");
            }

            service.dismiss_popup(id);
        });
    }

    pub(super) fn invoke_notification_default_action(&self, id: u32) {
        let Some(service) = self.services.notification.clone() else {
            tracing::info!(
                "Cannot invoke default notification action because notification service is unavailable"
            );
            return;
        };

        relm4::spawn(async move {
            if let Some(notification) =
                crate::notifications::actions::notification_by_id(service.as_ref(), id)
            {
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
            } else {
                tracing::info!(id, "Default notification action target disappeared");
            }

            service.dismiss_popup(id);
        });
    }
}

#[cfg(test)]
#[path = "notification_overlays_test.rs"]
mod tests;
