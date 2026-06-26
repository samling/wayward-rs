use super::*;

fn parse_document(contents: &str) -> toml_edit::DocumentMut {
    contents.parse::<toml_edit::DocumentMut>().unwrap()
}

fn bar_names(document: &toml_edit::DocumentMut) -> Vec<String> {
    document["bars"]
        .as_array_of_tables()
        .unwrap()
        .iter()
        .map(|bar| {
            bar.get("name")
                .and_then(|item| item.as_str())
                .unwrap()
                .to_string()
        })
        .collect()
}

#[test]
fn config_accepts_notification_monitor() {
    let config: AppConfig = toml::from_str(
        r#"
[notifications]
monitor = "DP-1"

[[bars]]
start = []
center = []
end = []
"#,
    )
    .unwrap();

    assert_eq!(config.notifications.monitor.as_deref(), Some("DP-1"));
}

#[test]
fn config_defaults_notification_monitor_to_focused_monitor() {
    let config: AppConfig = toml::from_str(
        r#"
[[bars]]
start = []
center = []
end = []
"#,
    )
    .unwrap();

    assert_eq!(config.notifications.monitor, None);
}

#[test]
fn set_document_value_does_not_create_tables_when_resetting_missing_path() {
    let mut document = parse_document("");

    set_document_value(
        &mut document,
        &["widgets", "updates", "critical-patterns"],
        None,
    );

    assert!(document.get("widgets").is_none());
}

#[test]
fn set_document_value_removes_empty_tables_after_reset() {
    let mut document = parse_document(
        r#"
[widgets.updates]
critical-patterns = ["linux-*"]
"#,
    );

    set_document_value(
        &mut document,
        &["widgets", "updates", "critical-patterns"],
        None,
    );

    assert!(document.get("widgets").is_none());
}

fn config_with_notification_monitor(monitor: Option<&str>) -> AppConfig {
    AppConfig {
        notifications: NotificationConfig {
            monitor: monitor.map(ToOwned::to_owned),
        },
        ..AppConfig::default()
    }
}

fn config_with_theme(theme: Option<&str>) -> AppConfig {
    AppConfig {
        theme: theme.map(ToOwned::to_owned),
        ..AppConfig::default()
    }
}

#[test]
fn config_changes_detects_noop_reload() {
    let previous = AppConfig::default();
    let next = AppConfig::default();

    assert_eq!(
        ConfigChanges::between(&previous, &next),
        ConfigChanges::default()
    );
    assert!(!ConfigChanges::between(&previous, &next).has_changes());
}

#[test]
fn config_changes_detects_notification_domain() {
    let previous = config_with_notification_monitor(None);
    let next = config_with_notification_monitor(Some("DP-1"));

    let changes = ConfigChanges::between(&previous, &next);

    assert!(changes.notifications_changed);
    assert!(!changes.bars_changed);
    assert!(!changes.style_changed);
    assert!(!changes.widgets_changed);
}

#[test]
fn config_changes_detects_style_domain() {
    let previous = config_with_theme(None);
    let next = config_with_theme(Some("dark"));

    let changes = ConfigChanges::between(&previous, &next);

    assert!(changes.style_changed);
    assert!(!changes.bars_changed);
    assert!(!changes.notifications_changed);
    assert!(!changes.widgets_changed);
}

#[test]
fn config_accepts_notification_style_controls() {
    let config: AppConfig = toml::from_str(
        r#"
    [style.notifications]
    body-font-weight = 500
    indicator-border-width = 2

    [[bars]]
    name = "bar"
    start = []
    center = []
    end = []
    "#,
    )
    .unwrap();

    use crate::config::style::StyleGroupExt;

    assert_eq!(
        config.style.notifications.integer("body-font-weight"),
        Some(500)
    );
    assert_eq!(
        config.style.notifications.integer("indicator-border-width"),
        Some(2)
    );
}

#[test]
fn set_bar_region_updates_only_named_bar_region() {
    let mut document = parse_document(
        r#"
[[bars]]
name = "top-bar"
edge = "top"
start = ["workspaces"]
center = ["clock"]
end = ["systray"]

[[bars]]
name = "other-bar"
edge = "bottom"
start = ["clock"]
center = []
end = []
"#,
    );

    set_bar_region_in_document(
        &mut document,
        "top-bar",
        BarRegionKey::Start,
        &["action_menu".to_string(), "workspaces".to_string()],
    )
    .unwrap();

    let bars = document["bars"].as_array_of_tables().unwrap();
    let top_bar = bars.get(0).unwrap();
    let other_bar = bars.get(1).unwrap();

    assert_eq!(
        top_bar["start"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>(),
        vec!["action_menu", "workspaces"]
    );
    assert_eq!(
        top_bar["center"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>(),
        vec!["clock"]
    );
    assert_eq!(
        other_bar["start"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>(),
        vec!["clock"]
    );
}

#[test]
fn add_bar_appends_empty_named_bar() {
    let mut document = parse_document(
        r#"
[[bars]]
name = "top-bar"
edge = "top"
start = ["workspaces"]
center = []
end = []
"#,
    );

    add_bar_to_document(&mut document, "side-bar").unwrap();

    assert_eq!(bar_names(&document), vec!["top-bar", "side-bar"]);

    let bars = document["bars"].as_array_of_tables().unwrap();
    let side_bar = bars.get(1).unwrap();

    assert_eq!(side_bar["edge"].as_str(), Some("top"));
    assert!(side_bar["start"].as_array().unwrap().is_empty());
    assert!(side_bar["center"].as_array().unwrap().is_empty());
    assert!(side_bar["end"].as_array().unwrap().is_empty());
}

#[test]
fn add_bar_rejects_duplicate_names() {
    let mut document = parse_document(
        r#"
[[bars]]
name = "top-bar"
edge = "top"
start = []
center = []
end = []
"#,
    );

    let error = add_bar_to_document(&mut document, "top-bar").unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
}

#[test]
fn remove_bar_removes_named_bar_only() {
    let mut document = parse_document(
        r#"
[[bars]]
name = "top-bar"
edge = "top"
start = ["workspaces"]
center = []
end = []

[[bars]]
name = "side-bar"
edge = "left"
start = ["clock"]
center = []
end = []
"#,
    );

    remove_bar_from_document(&mut document, "top-bar").unwrap();

    assert_eq!(bar_names(&document), vec!["side-bar"]);
}

#[test]
fn remove_bar_rejects_removing_last_bar() {
    let mut document = parse_document(
        r#"
[[bars]]
name = "top-bar"
edge = "top"
start = []
center = []
end = []
"#,
    );

    let error = remove_bar_from_document(&mut document, "top-bar").unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn set_bar_monitors_sets_and_removes_monitor_list() {
    let mut document = parse_document(
        r#"
[[bars]]
name = "top-bar"
edge = "top"
start = []
center = []
end = []
"#,
    );

    set_bar_monitors_in_document(
        &mut document,
        "top-bar",
        &["DP-1".to_string(), "DP-2".to_string()],
    )
    .unwrap();

    let bars = document["bars"].as_array_of_tables().unwrap();
    let top_bar = bars.get(0).unwrap();

    assert_eq!(
        top_bar["monitors"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>(),
        vec!["DP-1", "DP-2"]
    );

    set_bar_monitors_in_document(&mut document, "top-bar", &[]).unwrap();

    let bars = document["bars"].as_array_of_tables().unwrap();
    let top_bar = bars.get(0).unwrap();

    assert!(top_bar.get("monitors").is_none());
}

#[test]
fn set_bar_edge_updates_named_bar_edge() {
    let mut document = parse_document(
        r#"
    [[bars]]
    name = "top-bar"
    edge = "top"
    start = []
    center = []
    end = []

    [[bars]]
    name = "side-bar"
    edge = "left"
    start = []
    center = []
    end = []
    "#,
    );

    set_bar_edge_in_document(&mut document, "side-bar", "right").unwrap();

    let bars = document["bars"].as_array_of_tables().unwrap();

    assert_eq!(bars.get(0).unwrap()["edge"].as_str(), Some("top"));
    assert_eq!(bars.get(1).unwrap()["edge"].as_str(), Some("right"));
}

#[test]
fn rename_bar_updates_named_bar_only() {
    let mut document = parse_document(
        r#"
    [[bars]]
    name = "top-bar"
    edge = "top"
    start = []
    center = []
    end = []

    [[bars]]
    name = "side-bar"
    edge = "left"
    start = []
    center = []
    end = []
    "#,
    );

    rename_bar_in_document(&mut document, "side-bar", "left-bar").unwrap();

    assert_eq!(bar_names(&document), vec!["top-bar", "left-bar"]);
}

#[test]
fn rename_bar_rejects_duplicate_names() {
    let mut document = parse_document(
        r#"
    [[bars]]
    name = "top-bar"
    edge = "top"
    start = []
    center = []
    end = []

    [[bars]]
    name = "side-bar"
    edge = "left"
    start = []
    center = []
    end = []
    "#,
    );

    let error = rename_bar_in_document(&mut document, "side-bar", "top-bar").unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
}

#[test]
fn action_menu_sections_mut_creates_array_under_existing_table() {
    let mut document =
        parse_document("[widgets.action_menu]\n[widgets.action_menu.layout]\ncolumns = 3\n");
    {
        let sections = action_menu_sections_mut(&mut document).unwrap();
        let mut section = toml_edit::Table::new();
        section.insert("title", toml_edit::value("New section"));
        sections.push(section);
    }
    let rendered = document.to_string();
    assert!(rendered.contains("[[widgets.action_menu.sections]]"), "{rendered}");
    assert!(rendered.contains("title = \"New section\""), "{rendered}");
    // sibling tables are preserved
    assert!(rendered.contains("[widgets.action_menu.layout]"), "{rendered}");
}

#[test]
fn action_menu_sections_mut_creates_tables_when_missing() {
    let mut document = parse_document("");
    {
        let sections = action_menu_sections_mut(&mut document).unwrap();
        sections.push(toml_edit::Table::new());
    }
    assert!(
        document
            .to_string()
            .contains("[[widgets.action_menu.sections]]")
    );
}

#[test]
fn action_menu_sections_mut_materializes_defaults_when_absent() {
    let mut document = parse_document("");
    let sections = action_menu_sections_mut(&mut document).unwrap();
    assert!(
        !sections.is_empty(),
        "an absent sections key should materialize the built-in defaults"
    );
    assert!(sections.get(0).unwrap().contains_key("title"));
    assert!(sections.get(0).unwrap().contains_key("actions"));
}

#[test]
fn action_menu_sections_mut_preserves_explicit_empty() {
    let mut document = parse_document("[widgets.action_menu]\nsections = []\n");
    let sections = action_menu_sections_mut(&mut document).unwrap();
    assert!(
        sections.is_empty(),
        "an explicit empty list must not be refilled with defaults"
    );
}

#[test]
fn persist_empty_action_menu_sections_writes_inline_empty_array() {
    let mut document = parse_document("[[bars]]\nname = \"bar\"\n\n[widgets.action_menu]\n");
    persist_empty_action_menu_sections(&mut document);
    let rendered = document.to_string();
    assert!(rendered.contains("sections = []"), "{rendered}");

    // Round-trips as an empty configured list, not an absent key.
    let config: AppConfig = toml::from_str(&rendered).unwrap();
    let sections = config
        .widgets
        .get("action_menu")
        .and_then(|table| table.get("sections"))
        .and_then(|value| value.as_array());
    assert!(sections.is_some_and(|array| array.is_empty()));
}

#[test]
fn default_action_menu_section_tables_round_trip_from_widget_defaults() {
    let tables = default_action_menu_section_tables();
    // The widget ships at least one default section (Screenshot) with actions.
    assert!(!tables.is_empty());
    assert!(tables[0].contains_key("title"));
    assert!(tables[0].contains_key("actions"));
}

