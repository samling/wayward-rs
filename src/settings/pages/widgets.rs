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
                        label: "Blue light enable command",
                        path: &["widgets", "brightness", "blue-light-enable-command"],
                        value: string_value(widgets, "brightness", "blue-light-enable-command"),
                        default: "",
                    }),
                    SettingSpec::String(StringSpec {
                        label: "Blue light disable command",
                        path: &["widgets", "brightness", "blue-light-disable-command"],
                        value: string_value(widgets, "brightness", "blue-light-disable-command"),
                        default: "",
                    }),
                    SettingSpec::String(StringSpec {
                        label: "Blue light state command",
                        path: &["widgets", "brightness", "blue-light-state-command"],
                        value: string_value(widgets, "brightness", "blue-light-state-command"),
                        default: "",
                    }),
                ],
            },
        ],
    }
}

fn string_value(
    widgets: &BTreeMap<String, toml::value::Table>,
    widget: &str,
    key: &str,
) -> Option<String> {
    widgets
        .get(widget)
        .and_then(|table| table.get(key))
        .and_then(|value| value.as_str())
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
