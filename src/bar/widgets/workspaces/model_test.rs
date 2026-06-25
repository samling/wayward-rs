use super::*;

fn workspace(name: Option<&str>) -> WorkspaceSummary {
    WorkspaceSummary {
        id: 10,
        idx: 3,
        name: name.map(str::to_string),
        output: Some("DP-1".to_string()),
        is_active: true,
        is_focused: true,
        is_urgent: false,
    }
}

#[test]
fn formatted_label_supports_index() {
    assert_eq!(workspace(Some("code")).formatted_label("%I"), "3");
}

#[test]
fn formatted_label_supports_title() {
    assert_eq!(workspace(Some("code")).formatted_label("%T"), "code");
}

#[test]
fn formatted_label_uses_empty_title_when_name_is_missing() {
    assert_eq!(workspace(None).formatted_label("%I: %T"), "3: ");
}

#[test]
fn formatted_label_supports_composite_formats() {
    assert_eq!(workspace(Some("code")).formatted_label("%I: %T"), "3: code");
}

#[test]
fn formatted_label_supports_literal_percent() {
    assert_eq!(workspace(Some("code")).formatted_label("%%%I"), "%3");
}

#[test]
fn formatted_label_preserves_unknown_tokens() {
    assert_eq!(workspace(Some("code")).formatted_label("%X %I"), "%X 3");
}

#[test]
fn formatted_label_preserves_trailing_percent() {
    assert_eq!(workspace(Some("code")).formatted_label("%I%"), "3%");
}
