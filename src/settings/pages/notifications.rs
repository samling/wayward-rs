use crate::config::{StyleConfig, variables::style_setting_sections};

use super::super::spec::SettingsSectionSpec;

pub(crate) fn sections() -> Vec<&'static str> {
    style_setting_sections()
}

pub(crate) fn section_spec(section: &'static str, style: &StyleConfig) -> SettingsSectionSpec {
    super::style_sections::section(section, style)
}
