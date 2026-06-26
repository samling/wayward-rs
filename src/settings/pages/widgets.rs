use std::collections::BTreeMap;

use crate::settings_spec::SettingsSectionSpec;

pub(crate) fn config_sections(
    config_key: &str,
    widgets: &BTreeMap<String, toml::value::Table>,
) -> Vec<SettingsSectionSpec> {
    let Some(widget) = crate::bar::registry::widget_by_id(config_key) else {
        return Vec::new();
    };

    let empty = toml::value::Table::new();
    let table = widgets.get(config_key).unwrap_or(&empty);
    widget.settings_sections(table)
}
