use std::collections::BTreeMap;

use super::super::spec::{
    SettingSpec, SettingsPageSpec, SettingsSectionSpec, StringListSpec, StringSpec,
};

pub(crate) fn page(widgets: &BTreeMap<String, toml::value::Table>) -> SettingsPageSpec {
    SettingsPageSpec {
        title: "Widgets".to_string(),
        sections: vec![
            SettingsSectionSpec {
                title: "Updates".to_string(),
                settings: vec![
                    SettingSpec::StringList(StringListSpec {
                        label: "Critical patterns",
                        path: &["widgets", "updates", "critical-patterns"],
                        value: string_list_value(widgets, "updates", "critical-patterns"),
                        default: &[],
                    }),
                    SettingSpec::StringList(StringListSpec {
                        label: "Warning patterns",
                        path: &["widgets", "updates", "warning-patterns"],
                        value: string_list_value(widgets, "updates", "warning-patterns"),
                        default: &[],
                    }),
                ],
            },
            SettingsSectionSpec {
                title: "Brightness".to_string(),
                settings: vec![
                    SettingSpec::String(StringSpec {
                        label: "sunsetr automatic preset",
                        path: &["widgets", "brightness", "sunsetr", "automatic-preset"],
                        value: nested_string_value(
                            widgets,
                            "brightness",
                            &["sunsetr", "automatic-preset"],
                        ),
                        default: "default",
                    }),
                    SettingSpec::String(StringSpec {
                        label: "sunsetr paused preset",
                        path: &["widgets", "brightness", "sunsetr", "paused-preset"],
                        value: nested_string_value(
                            widgets,
                            "brightness",
                            &["sunsetr", "paused-preset"],
                        ),
                        default: "day",
                    }),
                ],
            },
        ],
    }
}

fn nested_string_value(
    widgets: &BTreeMap<String, toml::value::Table>,
    widget: &str,
    path: &[&str],
) -> Option<String> {
    let mut value = widgets.get(widget)?;

    for key in &path[..path.len().saturating_sub(1)] {
        value = value.get(*key)?.as_table()?;
    }

    value
        .get(path.last().copied()?)?
        .as_str()
        .map(ToOwned::to_owned)
}

fn string_list_value(
    widgets: &BTreeMap<String, toml::value::Table>,
    widget: &str,
    key: &str,
) -> Option<Vec<String>> {
    widgets
        .get(widget)
        .and_then(|table| table.get(key))
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str())
                .map(ToOwned::to_owned)
                .collect()
        })
}
