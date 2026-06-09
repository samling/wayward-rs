use crate::config::StyleConfig;

use super::super::spec::{NumberSpec, SettingSpec, SettingsPageSpec, SettingsSectionSpec};

pub(crate) fn page(style: &StyleConfig) -> SettingsPageSpec {
    SettingsPageSpec {
        title: "Notifications",
        sections: vec![SettingsSectionSpec {
            title: "Notification cards",
            settings: vec![
                SettingSpec::Number(NumberSpec {
                    label: "Body font weight",
                    path: &["style", "notifications", "body_font_weight"],
                    value: style.notifications.body_font_weight,
                    default: 500,
                    min: 100.0,
                    max: 900.0,
                    step: 50.0,
                }),
                SettingSpec::Number(NumberSpec {
                    label: "Normal border width",
                    path: &["style", "notifications", "normal_border_width_px"],
                    value: style.notifications.normal_border_width_px,
                    default: 0,
                    min: 0.0,
                    max: 8.0,
                    step: 1.0,
                }),
            ],
        }],
    }
}