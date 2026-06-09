use crate::config::{StyleConfig, style::StyleGroupExt};

use super::super::spec::{
    NumberSpec, SettingSpec, SettingsSectionSpec, StringSpec
};

pub(crate) fn section(style: &StyleConfig) -> SettingsSectionSpec {
    let bar = &style.bar;

    SettingsSectionSpec { 
        title: "Bar",
        settings: vec![
            SettingSpec::String(StringSpec {
                label: "Font family",
                path: &["style", "bar", "font-family"],
                value: bar.string("font-family"),
                default: "Adwaita Sans",
            })
        ]
    }
}