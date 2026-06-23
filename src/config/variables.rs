mod specs;

use super::style::{StyleConfig, StyleGroupConfig, StyleGroupExt};

pub(crate) trait CssVariables {
    fn write_css_variables(&self, css: &mut String);
}

#[derive(Clone, Copy)]
pub(super) enum CssValueKind {
    Integer {
        unit: &'static str,
    },
    String {
        quoted: bool,
    },
    Bool {
        true_value: &'static str,
        false_value: &'static str,
    },
}

#[derive(Clone, Copy)]
pub(crate) enum SettingUiSpec {
    Number {
        label: &'static str,
        default: u16,
        min: f64,
        max: f64,
        step: f64,
    },
    Toggle {
        label: &'static str,
        default: bool,
    },
    String {
        label: &'static str,
        default: &'static str,
    },
    Color {
        label: &'static str,
        default: &'static str,
    },
}

impl SettingUiSpec {
    fn integer_default(self) -> Option<u16> {
        match self {
            Self::Number { default, .. } => Some(default),
            _ => None,
        }
    }

    fn bool_default(self) -> Option<bool> {
        match self {
            Self::Toggle { default, .. } => Some(default),
            _ => None,
        }
    }

    fn string_default(self) -> Option<&'static str> {
        match self {
            Self::String { default, .. } | Self::Color { default, .. } => Some(default),
            _ => None,
        }
    }
}

pub(crate) struct StyleSettingSpec {
    pub(crate) section: &'static str,
    pub(crate) group: &'static str,
    pub(crate) key: &'static str,
    pub(crate) path: &'static [&'static str],
    pub(crate) setting: Option<SettingUiSpec>,
    variable: &'static str,
    css_kind: CssValueKind,
}

impl StyleSettingSpec {
    pub(crate) fn palette_fallback_key(&self) -> Option<&'static str> {
        match self.variable {
            "--bar-background-color" => Some("surface"),
            "--bar-color" => Some("on-surface"),
            "--workspace-active-color" => Some("on-surface"),
            "--workspace-focused-color" => Some("primary"),
            "--workspace-urgent-color" => Some("error"),
            "--workspace-indicator-background-color" => Some("primary-container"),
            "--workspace-status-color" => Some("tertiary"),
            "--surface-background-color" => Some("surface"),
            "--surface-border-color" => Some("outline"),
            "--surface-color" => Some("on-surface"),
            "--dropdown-header-background-color" => Some("surface-container-low"),
            "--dropdown-header-border-color" => Some("outline-variant"),
            "--dropdown-empty-background-color" => Some("surface-container-low"),
            "--dropdown-empty-border-color" => Some("outline-variant"),
            "--battery-meter-fill-color" => Some("primary"),
            "--battery-detail-background-color" => Some("surface-container"),
            "--battery-detail-border-color" => Some("outline-variant"),
            "--battery-profile-border-color" => Some("outline"),
            "--battery-profile-hover-background-color" => Some("surface-container-high"),
            "--battery-profile-hover-border-color" => Some("primary"),
            "--battery-profile-active-background-color" => Some("primary-container"),
            "--battery-profile-active-border-color" => Some("primary"),
            "--systray-menu-hover-background-color" => Some("surface-container-high"),
            "--osd-level-track-color" => Some("surface-container-high"),
            "--osd-level-fill-color" => Some("primary"),
            "--osd-brightness-level-fill-color" => Some("tertiary"),
            "--osd-volume-level-fill-color" => Some("primary"),
            "--osd-muted-level-fill-color" => Some("error"),
            "--notification-row-background-color" => Some("surface-container"),
            "--notification-row-hover-background-color" => Some("surface-container-high"),
            "--notification-row-border-color" => Some("outline"),
            "--notification-row-hover-border-color" => Some("outline"),
            "--notification-row-low-border-color" => Some("outline"),
            "--notification-row-normal-border-color" => Some("primary"),
            "--notification-row-critical-border-color" => Some("error"),
            "--notification-toast-border-color" => Some("outline"),
            "--notification-list-action-border-color" => Some("outline"),
            "--notification-action-border-color" => Some("outline"),
            "--notification-clear-all-border-color" => Some("outline"),
            "--notification-clear-all-hover-border-color" => Some("error"),
            "--notification-list-action-hover-background-color" => Some("surface-container-high"),
            "--notification-list-action-hover-border-color" => Some("primary"),
            "--notification-toast-critical-border-color" => Some("error"),
            "--notification-button-border-color" => Some("outline"),
            "--notification-button-hover-background-color" => Some("surface-container-high"),
            "--notification-button-hover-border-color" => Some("primary"),
            "--action-menu-section-background-color" => Some("surface-container-low"),
            "--action-menu-section-border-color" => Some("outline-variant"),
            "--dropdown-section-title-color" => Some("on-surface-variant"),
            "--action-menu-button-background-color" => Some("secondary-container"),
            "--action-menu-button-border-color" => Some("outline-variant"),
            "--action-menu-button-hover-background-color" => Some("surface-container-high"),
            "--action-menu-button-hover-border-color" => Some("secondary"),
            "--settings-background-color" => Some("surface-container-lowest"),
            "--settings-color" => Some("on-surface"),
            "--settings-titlebar-background-color" => Some("surface-container"),
            "--settings-titlebar-border-color" => Some("outline-variant"),
            "--settings-sidebar-background-color" => Some("surface-container-low"),
            "--settings-sidebar-active-background-color" => Some("secondary-container"),
            "--settings-group-background-color" => Some("surface-container"),
            "--settings-row-label-color" => Some("on-surface"),
            "--settings-row-value-color" => Some("on-surface-variant"),
            "--settings-color-error" => Some("error"),
            "--settings-token-background-color" => Some("surface-container-high"),
            "--settings-token-drop-border-color" => Some("primary"),
            "--settings-token-invalid-color" => Some("on-error-container"),
            "--settings-token-invalid-border-color" => Some("error"),
            "--settings-token-drop-background-color" => Some("primary-container"),
            _ => None,
        }
    }
}

pub(crate) fn palette_color_default(key: &str) -> Option<&'static str> {
    specs::style_settings().find_map(|spec| {
        if spec.group != "palette" || spec.key != key {
            return None;
        }

        match spec.setting {
            Some(SettingUiSpec::Color { default, .. }) => Some(default),
            _ => None,
        }
    })
}

impl CssVariables for StyleConfig {
    fn write_css_variables(&self, css: &mut String) {
        for spec in specs::style_settings() {
            let Some(group) = self.group(spec.group) else {
                continue;
            };

            write_mapped_css_variable(css, group, spec);
        }
    }
}

pub(crate) fn style_setting_sections() -> Vec<&'static str> {
    let mut sections = Vec::new();

    for spec in specs::style_settings() {
        if spec.setting.is_some() && !sections.contains(&spec.section) {
            sections.push(spec.section);
        }
    }

    sections
}

pub(crate) fn settings_for_section(
    section: &'static str,
) -> impl Iterator<Item = &'static StyleSettingSpec> {
    specs::style_settings().filter(move |spec| spec.section == section && spec.setting.is_some())
}

fn write_mapped_css_variable(css: &mut String, group: &StyleGroupConfig, spec: &StyleSettingSpec) {
    let should_write_default = should_write_default(spec);

    match spec.css_kind {
        CssValueKind::Integer { unit } => {
            let value = group.integer(spec.key).or_else(|| {
                should_write_default
                    .then(|| spec.setting.and_then(SettingUiSpec::integer_default))
                    .flatten()
            });

            if let Some(value) = value {
                write_css_variable(css, spec.variable, value, unit);
            }
        }
        CssValueKind::String { quoted } => {
            let value = group.string(spec.key).or_else(|| {
                should_write_default
                    .then(|| {
                        spec.setting
                            .and_then(SettingUiSpec::string_default)
                            .map(str::to_string)
                    })
                    .flatten()
            });

            if let Some(value) = value {
                let value = if quoted {
                    format!("\"{value}\"")
                } else {
                    value
                };
                write_css_variable(css, spec.variable, value, "");
            }
        }
        CssValueKind::Bool {
            true_value,
            false_value,
        } => {
            let value = group.bool(spec.key).or_else(|| {
                should_write_default
                    .then(|| spec.setting.and_then(SettingUiSpec::bool_default))
                    .flatten()
            });

            if let Some(value) = value {
                let value = if value { true_value } else { false_value };
                write_css_variable(css, spec.variable, value, "");
            }
        }
    }
}

fn should_write_default(spec: &StyleSettingSpec) -> bool {
    if spec.group == "palette" {
        return true;
    }

    if spec.group == "bar" && matches!(spec.key, "background-color" | "color") {
        return false;
    }

    if spec.group == "bar" {
        return true;
    }

    !spec.key.starts_with("widget-") && !matches!(spec.setting, Some(SettingUiSpec::Color { .. }))
}

fn write_css_variable<T: std::fmt::Display>(css: &mut String, name: &str, value: T, unit: &str) {
    css.push_str(&format!("  {name}: {value}{unit};\n"));
}
