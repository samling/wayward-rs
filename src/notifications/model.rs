use chrono::{DateTime, Utc};
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
    pub(crate) image_path: Option<String>,
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
    pub(crate) image_path: Option<String>,
    pub(crate) summary: String,
    pub(crate) body: Option<String>,
    pub(crate) urgency: Urgency,
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) actions: Vec<NotificationAction>,
    pub(crate) default_action: Option<NotificationAction>,
}

impl NotificationToast {
    pub(crate) fn from_notification(notification: &Notification) -> Self {
        tracing::debug!(
            id = notification.id,
            app_name = ?notification.app_name.get(),
            app_icon = ?notification.app_icon.get(),
            image_path = ?notification.image_path.get(),
            desktop_entry = ?notification.desktop_entry.get(),
            "notification icon fields"
        );
        Self::from_fields(NotificationToastFields {
            id: notification.id,
            app_name: notification.app_name.get(),
            app_icon: notification.app_icon.get(),
            image_path: notification.image_path.get(),
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
            default_action: notification
                .default_action
                .get()
                .map(|action| NotificationAction {
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
            image_path: fields.image_path,
            summary: fields.summary,
            body: clean_body(fields.body),
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
}

fn display_or_fallback(value: Option<String>, fallback: &str) -> String {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn clean_body(body: Option<String>) -> Option<String> {
    body.and_then(|body| {
        let body = strip_leading_origin_link(&body).trim().to_string();

        if body.is_empty() {
            None
        } else {
            Some(body)
        }
    })
}

fn strip_leading_origin_link(body: &str) -> &str {
    let Some(rest) = body.strip_prefix("<a href=\"") else {
        return body;
    };

    let Some((href, rest)) = rest.split_once("\">") else {
        return body;
    };

    let Some((label, rest)) = rest.split_once("</a>") else {
        return body;
    };

    if !origin_label_matches_href(label, href) {
        return body;
    }

    rest.trim_start_matches(['\r', '\n'])
}

fn origin_label_matches_href(label: &str, href: &str) -> bool {
    let Some(host) = href_host(href) else {
        return false;
    };

    label == host || label.strip_prefix("www.") == Some(host) || host.strip_prefix("www.") == Some(label)
}

fn href_host(href: &str) -> Option<&str> {
    let rest = href
        .strip_prefix("https://")
        .or_else(|| href.strip_prefix("http://"))?;

    rest.split(['/', '?', '#']).next()
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

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use super::*;

    #[test]
    fn from_fields_uses_fallbacks_for_blank_app_data() {
        let toast = NotificationToast::from_fields(fields(
            1,
            Some("  ".to_string()),
            Some("".to_string()),
            "Summary",
            "2026-06-04T12:00:00Z",
        ));

        assert_eq!(toast.app_name, FALLBACK_APP_NAME);
        assert_eq!(toast.app_icon, FALLBACK_ICON_NAME);
    }

    #[test]
    fn visible_actions_excludes_default_action() {
        let mut toast = NotificationToast::from_fields(fields(
            1,
            Some("Mail".to_string()),
            Some("mail-unread-symbolic".to_string()),
            "Message",
            "2026-06-04T12:00:00Z",
        ));

        toast.actions = vec![
            action("default", "Open"),
            action("reply", "Reply"),
            action("archive", "Archive"),
        ];

        assert_eq!(
            toast.visible_actions(),
            vec![action("reply", "Reply"), action("archive", "Archive")]
        );
    }

    #[test]
    fn urgency_class_matches_urgency() {
        let mut toast = NotificationToast::from_fields(fields(
            1,
            Some("Calendar".to_string()),
            Some("x-office-calendar-symbolic".to_string()),
            "Meeting",
            "2026-06-04T12:00:00Z",
        ));

        toast.urgency = Urgency::Critical;

        assert_eq!(toast.urgency_class(), "critical");
    }

    #[test]
    fn newest_first_orders_by_timestamp_then_id() {
        let oldest = NotificationToast::from_fields(fields(
            1,
            Some("App".to_string()),
            Some("dialog-information-symbolic".to_string()),
            "Old",
            "2026-06-04T12:00:00Z",
        ));
        let newer_low_id = NotificationToast::from_fields(fields(
            2,
            Some("App".to_string()),
            Some("dialog-information-symbolic".to_string()),
            "Newer low id",
            "2026-06-04T13:00:00Z",
        ));
        let newer_high_id = NotificationToast::from_fields(fields(
            3,
            Some("App".to_string()),
            Some("dialog-information-symbolic".to_string()),
            "Newer high id",
            "2026-06-04T13:00:00Z",
        ));

        let result = newest_first(vec![oldest, newer_low_id, newer_high_id]);

        assert_eq!(
            result.iter().map(|toast| toast.id).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
    }

    fn fields(
        id: u32,
        app_name: Option<String>,
        app_icon: Option<String>,
        summary: &str,
        timestamp: &str,
    ) -> NotificationToastFields {
        NotificationToastFields {
            id,
            app_name,
            app_icon,
            image_path: None,
            summary: summary.to_string(),
            body: Some("Body".to_string()),
            urgency: Urgency::Normal,
            timestamp: DateTime::parse_from_rfc3339(timestamp)
                .unwrap()
                .with_timezone(&Utc),
            actions: Vec::new(),
            default_action: None,
        }
    }

    fn action(id: &str, label: &str) -> NotificationAction {
        NotificationAction {
            id: id.to_string(),
            label: label.to_string(),
        }
    }
}
