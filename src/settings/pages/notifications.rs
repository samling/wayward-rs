use crate::config::{StyleConfig, style::StyleGroupExt};

use super::super::spec::{
    NumberSpec, SettingSpec, SettingsPageSpec, SettingsSectionSpec, StringSpec, ToggleSpec,
};

pub(crate) fn page(style: &StyleConfig) -> SettingsPageSpec {
    let notifications = &style.notifications;

    SettingsPageSpec {
        title: "Notifications",
        sections: vec![SettingsSectionSpec {
            title: "Notification cards",
            settings: vec![
                SettingSpec::Number(NumberSpec {
                    label: "Body font weight",
                    path: &["style", "notifications", "body-font-weight"],
                    value: notifications.integer("body-font-weight"),
                    default: 500,
                    min: 100.0,
                    max: 900.0,
                    step: 50.0,
                }),
                SettingSpec::Number(NumberSpec {
                    label: "Normal border width",
                    path: &["style", "notifications", "normal-border-width"],
                    value: notifications.integer("normal-border-width"),
                    default: 0,
                    min: 0.0,
                    max: 8.0,
                    step: 1.0,
                }),
                SettingSpec::Toggle(ToggleSpec {
                    label: "Hide scrollbar",
                    path: &["style", "notifications", "hide-scrollbar"],
                    value: notifications.bool("hide-scrollbar"),
                    default: true,
                }),
                SettingSpec::String(StringSpec {
                    label: "Font family",
                    path: &["style", "notifications", "font-family"],
                    value: notifications.string("font-family"),
                    default: "Adwaita Sans",
                }),
                SettingSpec::Number(NumberSpec {
                    label: "List icon size",
                    path: &["style", "notifications", "list-icon-size"],
                    value: notifications.integer("list-icon-size"),
                    default: 30,
                    min: 16.0,
                    max: 64.0,
                    step: 1.0,
                }),
            ],
        }],
    }
}
