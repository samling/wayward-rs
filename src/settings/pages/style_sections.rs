use crate::config::{
    StyleConfig,
    style::StyleGroupExt,
    variables::{SettingUiSpec, settings_for_section},
};

use super::super::spec::{
    ColorSettingRole, ColorSpec, NumberSpec, SettingSpec, SettingsSectionSpec, StringSpec,
    ToggleSpec,
};

pub(crate) fn section(section_name: &'static str, style: &StyleConfig) -> SettingsSectionSpec {
    let settings = settings_for_section(section_name)
        .filter_map(|spec| {
            let group = style.group(spec.group);
            let setting = spec.setting?;

            match setting {
                SettingUiSpec::Number {
                    label,
                    default,
                    min,
                    max,
                    step,
                } => Some(SettingSpec::Number(NumberSpec {
                    label,
                    path: spec.path,
                    value: group.and_then(|group| group.integer(spec.key)),
                    default,
                    min,
                    max,
                    step,
                })),
                SettingUiSpec::Toggle { label, default } => Some(SettingSpec::Toggle(ToggleSpec {
                    label,
                    path: spec.path,
                    value: group.and_then(|group| group.bool(spec.key)),
                    default,
                })),
                SettingUiSpec::String { label, default } => Some(SettingSpec::String(StringSpec {
                    label,
                    path: spec.path,
                    value: group.and_then(|group| group.string(spec.key)),
                    default,
                })),
                SettingUiSpec::Color { label, default } => Some(SettingSpec::Color(ColorSpec {
                    label,
                    path: spec.path,
                    value: group.and_then(|group| group.string(spec.key)),
                    default,
                    role: color_setting_role(spec),
                })),
            }
        })
        .collect();

    SettingsSectionSpec {
        title: section_name.to_string(),
        settings,
    }
}

fn color_setting_role(spec: &crate::config::variables::StyleSettingSpec) -> ColorSettingRole {
    if spec.group == "palette" {
        ColorSettingRole::Palette
    } else if spec.group == "bar" && spec.key.starts_with("widget-") {
        ColorSettingRole::Default
    } else {
        ColorSettingRole::Override
    }
}
