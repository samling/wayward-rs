use crate::config::StyleConfig;

use super::super::spec::SettingsPageSpec;

pub(crate) fn page(style: &StyleConfig) -> SettingsPageSpec {
    SettingsPageSpec {
        title: "Appearance",
        sections: vec![
            super::bar::section(style),
            super::style_sections::section("Notification cards", "notifications", style),
        ],
    }
}
