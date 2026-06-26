use super::*;
use crate::config::ConfigValue;

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
