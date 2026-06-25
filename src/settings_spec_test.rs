use super::*;

#[test]
fn string_list_spec_displays_values_as_comma_separated_text() {
    let setting = StringListSpec {
        label: "Critical patterns",
        description: None,
        path: &["widgets", "updates", "critical-patterns"],
        value: Some(vec!["linux-*".to_string(), "pacman".to_string()]),
        default: &[],
    };

    assert_eq!(setting.display_value(), "linux-*, pacman");
}

#[test]
fn string_list_spec_parses_comma_separated_text_for_config() {
    let setting = StringListSpec {
        label: "Critical patterns",
        description: None,
        path: &["widgets", "updates", "critical-patterns"],
        value: None,
        default: &[],
    };

    assert_eq!(
        setting.value_for_config(" linux-* , pacman,, ".to_string()),
        ConfigValue::StringList(vec!["linux-*".to_string(), "pacman".to_string()])
    );
}
