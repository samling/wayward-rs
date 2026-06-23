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
    spec.group == "bar" || !spec.key.starts_with("widget-")
}

fn write_css_variable<T: std::fmt::Display>(css: &mut String, name: &str, value: T, unit: &str) {
    css.push_str(&format!("  {name}: {value}{unit};\n"));
}
