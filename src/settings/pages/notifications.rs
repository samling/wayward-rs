use crate::config::{StyleConfig, variables::style_setting_sections};

use super::super::spec::SettingsPageSpec;

pub(crate) fn page(style: &StyleConfig) -> SettingsPageSpec {
    SettingsPageSpec {
        title: "Appearance".to_string(),
        sections: style_setting_sections()
            .into_iter()
            .map(|section| super::style_sections::section(section, style))
            .collect(),
    }
}
