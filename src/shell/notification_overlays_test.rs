use super::*;

#[test]
fn notification_target_uses_configured_monitor_when_available() {
    assert_eq!(
        notification_target_connector(Some("DP-1"), Some("HDMI-A-1"), &["DP-1", "HDMI-A-1"],),
        Some("DP-1")
    );
}

#[test]
fn notification_target_falls_back_to_focused_monitor_when_configured_monitor_is_missing() {
    assert_eq!(
        notification_target_connector(Some("DP-2"), Some("HDMI-A-1"), &["DP-1", "HDMI-A-1"],),
        Some("HDMI-A-1")
    );
}

#[test]
fn notification_target_uses_focused_monitor_without_configured_monitor() {
    assert_eq!(
        notification_target_connector(None, Some("HDMI-A-1"), &["DP-1", "HDMI-A-1"]),
        Some("HDMI-A-1")
    );
}

#[test]
fn notification_target_returns_none_without_configured_or_focused_monitor() {
    assert_eq!(notification_target_connector(None, None, &["DP-1"]), None);
}
