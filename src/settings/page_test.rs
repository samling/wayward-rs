use super::*;
use crate::config::ConfigValue;
use crate::settings_spec::SettingSpec;

#[test]
fn build_page_for_updates_widget_has_config_then_style() {
    let item = crate::settings::nav::find_item("updates").unwrap();
    let config = SettingsConfig {
        style: StyleConfig::default(),
        widgets: BTreeMap::new(),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };
    let page = build_page(item, &config).unwrap();
    assert_eq!(page.title, "Updates");
    assert_eq!(page.sections.first().unwrap().title, "Config");
    assert_eq!(page.sections.last().unwrap().title, "Style");
}

#[test]
fn build_page_for_clock_widget_has_config_and_style() {
    let item = crate::settings::nav::find_item("clock").unwrap();
    let config = SettingsConfig {
        style: StyleConfig::default(),
        widgets: BTreeMap::new(),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };
    let page = build_page(item, &config).unwrap();
    // The clock widget now declares a Config section (Time format) via settings_sections,
    // and build_page appends the Style section.
    assert_eq!(page.sections.len(), 2);
    assert_eq!(page.sections[0].title, "Config");
    assert_eq!(page.sections[1].title, "Style");
}

#[test]
fn settings_config_applies_widget_string_value() {
    let mut config = SettingsConfig {
        style: StyleConfig::default(),
        widgets: BTreeMap::new(),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };

    assert!(config.apply_config_value(
        &["widgets", "example", "label"],
        Some(&ConfigValue::String("Hello".to_string())),
    ));

    let value = config
        .widgets
        .get("example")
        .and_then(|table| table.get("label"))
        .and_then(|value| value.as_str());

    assert_eq!(value, Some("Hello"));
}

#[test]
fn settings_config_applies_widget_string_list_value() {
    let mut config = SettingsConfig {
        style: StyleConfig::default(),
        widgets: BTreeMap::new(),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };

    assert!(config.apply_config_value(
        &["widgets", "updates", "critical-patterns"],
        Some(&ConfigValue::StringList(vec!["linux-*".to_string()])),
    ));

    let values = config
        .widgets
        .get("updates")
        .and_then(|table| table.get("critical-patterns"))
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>()
        });

    assert_eq!(values, Some(vec!["linux-*"]));
}

#[test]
fn settings_config_removes_empty_widget_table_after_reset() {
    let mut updates = toml::value::Table::new();
    updates.insert(
        "critical-patterns".to_string(),
        toml::Value::Array(vec![toml::Value::String("linux-*".to_string())]),
    );

    let mut config = SettingsConfig {
        style: StyleConfig::default(),
        widgets: BTreeMap::from([("updates".to_string(), updates)]),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };

    assert!(config.apply_config_value(&["widgets", "updates", "critical-patterns"], None));

    assert!(!config.widgets.contains_key("updates"));
}

#[test]
fn settings_config_applies_nested_widget_string_value() {
    let mut config = SettingsConfig {
        style: StyleConfig::default(),
        widgets: BTreeMap::new(),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };

    assert!(config.apply_config_value(
        &["widgets", "brightness", "sunsetr", "paused-preset"],
        Some(&ConfigValue::String("day".to_string())),
    ));

    let value = config
        .widgets
        .get("brightness")
        .and_then(|table| table.get("sunsetr"))
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("paused-preset"))
        .and_then(toml::Value::as_str);

    assert_eq!(value, Some("day"));
}

#[test]
fn settings_config_removes_empty_nested_widget_table_after_reset() {
    let mut sunsetr = toml::value::Table::new();
    sunsetr.insert(
        "paused-preset".to_string(),
        toml::Value::String("day".to_string()),
    );

    let mut brightness = toml::value::Table::new();
    brightness.insert("sunsetr".to_string(), toml::Value::Table(sunsetr));

    let mut config = SettingsConfig {
        style: StyleConfig::default(),
        widgets: BTreeMap::from([("brightness".to_string(), brightness)]),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };

    assert!(
        config.apply_config_value(&["widgets", "brightness", "sunsetr", "paused-preset"], None,)
    );

    assert!(!config.widgets.contains_key("brightness"));
}

#[test]
fn settings_config_applies_action_menu_section_field_to_defaults() {
    let mut config = SettingsConfig {
        style: StyleConfig::default(),
        widgets: BTreeMap::new(),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };

    assert!(config.apply_action_menu_section_field(
        0,
        "title",
        Some(&ConfigValue::String("Captures".to_string())),
    ));

    let title = config
        .widgets
        .get("action_menu")
        .and_then(|table| table.get("sections"))
        .and_then(toml::Value::as_array)
        .and_then(|sections| sections.first())
        .and_then(toml::Value::as_table)
        .and_then(|section| section.get("title"))
        .and_then(toml::Value::as_str);

    assert_eq!(title, Some("Captures"));
}

#[test]
fn settings_config_applies_action_menu_action_field() {
    let mut action = toml::value::Table::new();
    action.insert("label".to_string(), toml::Value::String("Old".to_string()));
    action.insert(
        "tooltip".to_string(),
        toml::Value::String("Remove me".to_string()),
    );

    let mut section = toml::value::Table::new();
    section.insert(
        "actions".to_string(),
        toml::Value::Array(vec![toml::Value::Table(action)]),
    );

    let mut action_menu = toml::value::Table::new();
    action_menu.insert(
        "sections".to_string(),
        toml::Value::Array(vec![toml::Value::Table(section)]),
    );

    let mut config = SettingsConfig {
        style: StyleConfig::default(),
        widgets: BTreeMap::from([("action_menu".to_string(), action_menu)]),
        bars: Vec::new(),
        available_monitors: Vec::new(),
    };

    assert!(config.apply_action_menu_action_field(
        0,
        0,
        "label",
        Some(&ConfigValue::String("New".to_string())),
    ));
    assert!(config.apply_action_menu_action_field(0, 0, "tooltip", None));

    let action = config
        .widgets
        .get("action_menu")
        .and_then(|table| table.get("sections"))
        .and_then(toml::Value::as_array)
        .and_then(|sections| sections.first())
        .and_then(toml::Value::as_table)
        .and_then(|section| section.get("actions"))
        .and_then(toml::Value::as_array)
        .and_then(|actions| actions.first())
        .and_then(toml::Value::as_table)
        .unwrap();

    assert_eq!(
        action.get("label").and_then(toml::Value::as_str),
        Some("New")
    );
    assert!(!action.contains_key("tooltip"));
}

#[test]
fn action_menu_config_sections_use_widget_owned_settings() {
    let mut panel = toml::value::Table::new();
    panel.insert("width".to_string(), toml::Value::Integer(320));

    let mut layout = toml::value::Table::new();
    layout.insert("columns".to_string(), toml::Value::Integer(4));

    let mut header = toml::value::Table::new();
    header.insert(
        "power-command".to_string(),
        toml::Value::String("systemctl poweroff".to_string()),
    );
    header.insert(
        "power-args".to_string(),
        toml::Value::Array(vec![toml::Value::String("--logout".to_string())]),
    );

    let mut action_menu = toml::value::Table::new();
    action_menu.insert("panel".to_string(), toml::Value::Table(panel));
    action_menu.insert("layout".to_string(), toml::Value::Table(layout));
    action_menu.insert("header".to_string(), toml::Value::Table(header));

    let widgets = BTreeMap::from([("action_menu".to_string(), action_menu)]);
    let sections = super::super::pages::widgets::config_sections("action_menu", &widgets);

    assert_eq!(
        sections
            .iter()
            .map(|section| section.title.as_str())
            .collect::<Vec<_>>(),
        vec!["Panel", "Layout", "Header"]
    );
    assert_eq!(sections[0].settings.len(), 1);
    assert_eq!(sections[1].settings.len(), 4);
    assert_eq!(sections[2].settings.len(), 2);

    let SettingSpec::Number(width) = &sections[0].settings[0] else {
        panic!("width setting");
    };
    assert_eq!(width.path, ["widgets", "action_menu", "panel", "width"]);
    assert_eq!(width.value, Some(320));
    assert_eq!(width.default, 268);

    let SettingSpec::Number(columns) = &sections[1].settings[0] else {
        panic!("columns setting");
    };
    assert_eq!(
        columns.path,
        ["widgets", "action_menu", "layout", "columns"]
    );
    assert_eq!(columns.value, Some(4));
    assert_eq!(columns.default, 3);

    let SettingSpec::String(power_command) = &sections[2].settings[0] else {
        panic!("power command setting");
    };
    assert_eq!(
        power_command.path,
        ["widgets", "action_menu", "header", "power-command"]
    );
    assert_eq!(power_command.value.as_deref(), Some("systemctl poweroff"));
    assert_eq!(power_command.default, "wlogout");
}
