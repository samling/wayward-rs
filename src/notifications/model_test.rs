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
        desktop_entry: None,
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
