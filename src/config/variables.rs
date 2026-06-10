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
    pub(crate) group: &'static str,
    pub(crate) key: &'static str,
    pub(crate) path: &'static [&'static str],
    pub(crate) setting: Option<SettingUiSpec>,
    variable: &'static str,
    css_kind: CssValueKind,
}

impl CssVariables for StyleConfig {
    fn write_css_variables(&self, css: &mut String) {
        for spec in specs::STYLE_SETTINGS {
            let Some(group) = self.group(spec.group) else {
                continue;
            };

            write_mapped_css_variable(css, group, spec);
        }
    }
}

pub(crate) fn settings_for_group(
    group: &'static str,
) -> impl Iterator<Item = &'static StyleSettingSpec> {
    specs::STYLE_SETTINGS
        .iter()
        .filter(move |spec| spec.group == group && spec.setting.is_some())
}

fn write_mapped_css_variable(css: &mut String, group: &StyleGroupConfig, spec: &StyleSettingSpec) {
    match spec.css_kind {
        CssValueKind::Integer { unit } => {
            let value = group
                .integer(spec.key)
                .or_else(|| spec.setting.and_then(SettingUiSpec::integer_default));

            if let Some(value) = value {
                write_css_variable(css, spec.variable, value, unit);
            }
        }
        CssValueKind::String { quoted } => {
            let value = group.string(spec.key).or_else(|| {
                spec.setting
                    .and_then(SettingUiSpec::string_default)
                    .map(str::to_string)
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
            let value = group
                .bool(spec.key)
                .or_else(|| spec.setting.and_then(SettingUiSpec::bool_default));

            if let Some(value) = value {
                let value = if value { true_value } else { false_value };
                write_css_variable(css, spec.variable, value, "");
            }
        }
    }
}

fn write_css_variable<T: std::fmt::Display>(css: &mut String, name: &str, value: T, unit: &str) {
    css.push_str(&format!("  {name}: {value}{unit};\n"));
}
