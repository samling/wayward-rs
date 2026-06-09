use crate::config::StyleConfig;

use super::super::spec::SettingsSectionSpec;

pub(crate) fn section(style: &StyleConfig) -> SettingsSectionSpec {
    super::style_sections::section("Bar", "bar", style)
}
