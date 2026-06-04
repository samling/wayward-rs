use chrono::{DateTime, Utc};
use tracing::subscriber::with_default;
use wayle_notification::{core::notification::Notification, types::Urgency};

const FALLBACK_APP_NAME: &str = "Application";
const FALLBACK_ICON_NAME: &str = "dialog-information-symbolic";
const DEFAULT_ACTION_ID: &str = "default";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotificationAction {
    pub(crate) id: String,
    pub(crate) label: String,
}

#[derive(Clone, Debug)]
pub(crate) struct NotificationToastFields {
    pub(crate) id: u32,
    pub(crate) app_name: Option<String>,
    pub(crate) app_icon: Option<String>,
    pub(crate) summary: String,
    pub(crate) body: Option<String>,
    pub(crate) urgency: Urgency,
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) actions: Vec<NotificationAction>,
    pub(crate) default_action: Option<NotificationAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotificationToast {
    pub(crate) id: u32,
    pub(crate) app_name: String,
    pub(crate) app_icon: String,
    pub(crate) summary: String,
    pub(crate) body: Option<String>,
    pub(crate) urgency: Urgency,
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) actions: Vec<NotificationAction>,
    pub(crate) default_action: Option<NotificationAction>,
}

impl NotificationToast {
    pub(crate) fn from_notification(notification: &Notification) -> Self {
        Self::from_fields(NotificationToastFields {
            id: notification.id,
            app_name: notification.app_name.get(),
            app_icon: notification.app_icon.get(),
            summary: notification.summary.get(),
            body: notification.body.get(),
            actions: notification
                .actions
                .get()
                .into_iter()
                .map(|action| NotificationAction {
                    id: action.id,
                    label: action.label,
                })
                .collect(),
            default_action: notification.default_action.get().map(|action| NotificationAction {
                id: action.id,
                label: action.label,
            }),
            urgency: notification.urgency.get(),
            timestamp: notification.timestamp.get(),
        })
    }

    pub(crate) fn from_fields(fields: NotificationToastFields) -> Self {
        Self {
            id: fields.id,
            app_name: display_or_fallback(fields.app_name, FALLBACK_APP_NAME),
            app_icon: display_or_fallback(fields.app_icon, FALLBACK_ICON_NAME),
            summary: fields.summary,
            body: fields.body,
            urgency: fields.urgency,
            timestamp: fields.timestamp,
            actions: fields.actions,
            default_action: fields.default_action,
        }
    }

    pub(crate) fn visible_actions(&self) -> Vec<NotificationAction> {
        self.actions
            .iter()
            .filter(|action| action.id != DEFAULT_ACTION_ID)
            .cloned()
            .collect()
    }

    pub(crate) fn urgency_class(&self) -> &'static str {
        match self.urgency {
            Urgency::Low => "low",
            Urgency::Normal => "normal",
            Urgency::Critical => "critical",
        }
    }

    pub(crate) fn has_default_action(&self) -> bool {
        self.default_action.is_some()
    }

    pub(crate) fn newest_first(mut toasts: Vec<NotificationToast>) -> Vec<NotificationToast> {
        toasts.sort_by(|left, right| {
            right
                .timestamp
                .cmp(&left.timestamp)
                .then_with(|| right.id.cmp(&left.id))
        });
        toasts
    }
}

fn display_or_fallback(value: Option<String>, fallback: &str) -> String {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}