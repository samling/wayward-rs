use crate::config::{
    StyleConfig,
    style::StyleGroupExt,
    variables::{SettingUiSpec, settings_for_group},
};

use super::super::spec::{NumberSpec, SettingSpec, SettingsSectionSpec, StringSpec, ToggleSpec};

pub(crate) fn section(
    title: &'static str,
    group_name: &'static str,
    style: &StyleConfig,
) -> SettingsSectionSpec {
    let group = style.group(group_name);

    let settings = settings_for_group(group_name)
        .filter_map(|spec| {
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
            }
        })
        .collect();

    SettingsSectionSpec { title: title.to_string(), settings }
}
