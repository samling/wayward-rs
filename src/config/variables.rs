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
        default: ColorDefault,
    },
}

#[derive(Clone, Copy)]
pub(crate) enum ColorDefault {
    Literal(&'static str),
    Palette(&'static str),
}

impl ColorDefault {
    pub(crate) fn resolve(self, style: &StyleConfig) -> String {
        match self {
            Self::Literal(value) => value.to_string(),
            Self::Palette(key) => style
                .group("palette")
                .and_then(|group| group.string(key))
                .or_else(|| palette_color_default(key).map(str::to_string))
                .unwrap_or_default(),
        }
    }

    fn literal(self) -> Option<&'static str> {
        match self {
            Self::Literal(value) => Some(value),
            Self::Palette(_) => None,
        }
    }
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
            Self::String { default, .. } => Some(default),
            Self::Color { default, .. } => default.literal(),
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

pub(crate) fn palette_color_default(key: &str) -> Option<&'static str> {
    specs::style_settings().find_map(|spec| {
        if spec.group != "palette" || spec.key != key {
            return None;
        }

        match spec.setting {
            Some(SettingUiSpec::Color {
                default: ColorDefault::Literal(default),
                ..
            }) => Some(default),
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

#[cfg(test)]
mod tests {
    use super::{settings_for_section, style_setting_sections};

    #[test]
    fn floating_surfaces_section_is_removed() {
        assert!(!style_setting_sections().contains(&"Floating surfaces"));
    }

    #[test]
    fn osd_section_exposes_surface_settings() {
        let keys: Vec<&str> = settings_for_section("OSD").map(|spec| spec.key).collect();
        for key in ["background-color", "border-color", "border-radius", "color"] {
            assert!(keys.contains(&key), "OSD missing {key}");
        }
    }

    #[test]
    fn notification_cards_section_exposes_toast_surface_settings() {
        let keys: Vec<&str> =
            settings_for_section("Notification cards").map(|spec| spec.key).collect();
        for key in ["toast-background-color", "toast-border-radius", "toast-color", "toast-shadow"] {
            assert!(keys.contains(&key), "Notification cards missing {key}");
        }
    }
}
