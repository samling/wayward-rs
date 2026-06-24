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
        opacity_default: u16,
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

            write_mapped_css_variable(css, self, group, spec);
        }
    }
}

pub(crate) fn settings_for_section(
    section: &'static str,
) -> impl Iterator<Item = &'static StyleSettingSpec> {
    specs::style_settings().filter(move |spec| spec.section == section && spec.setting.is_some())
}

pub(crate) fn opacity_key(color_key: &str) -> String {
    match color_key.strip_suffix("color") {
        Some(prefix) => format!("{prefix}opacity"),
        None => format!("{color_key}-opacity"),
    }
}

fn resolve_token(token: &str, style: &StyleConfig) -> String {
    style
        .group("palette")
        .and_then(|group| group.string(token))
        .or_else(|| palette_color_default(token).map(str::to_string))
        .unwrap_or_else(|| token.to_string())
}

fn opacity_default_of(setting: SettingUiSpec) -> Option<u16> {
    match setting {
        SettingUiSpec::Color { opacity_default, .. } => Some(opacity_default),
        _ => None,
    }
}

fn write_mapped_css_variable(
    css: &mut String,
    style: &StyleConfig,
    group: &StyleGroupConfig,
    spec: &StyleSettingSpec,
) {
    if matches!(spec.setting, Some(SettingUiSpec::Color { .. })) {
        // Resolve a stored value (palette token name or literal) to a solid color string.
        let resolve = |raw: String, style: &StyleConfig| -> String {
            if super::color::parse_rgb(&raw).is_some() || raw.trim() == "transparent" {
                raw
            } else {
                resolve_token(&raw, style)
            }
        };

        if spec.group == "palette" {
            // Palette tokens always emit a solid color (configured value or default).
            let raw = group
                .string(spec.key)
                .or_else(|| spec.setting.and_then(SettingUiSpec::string_default).map(str::to_string));
            let Some(raw) = raw else { return };
            let resolved = resolve(raw, style);
            let value = super::color::solid_hex(&resolved).unwrap_or(resolved);
            write_css_variable(css, spec.variable, value, "");
            return;
        }

        let configured_color = group.string(spec.key);
        let configured_opacity = group.integer(&opacity_key(spec.key));

        let raw = match configured_color {
            Some(ref value) => value.clone(),
            None => match spec.setting {
                Some(SettingUiSpec::Color { default, .. }) => default.resolve(style),
                _ => return,
            },
        };
        let resolved = resolve(raw.clone(), style);
        // If no explicit opacity key, extract alpha from a user-supplied rgba color.
        // Only extract embedded alpha when the configured value is actually rgba; solid
        // hex or token strings must fall through to opacity_default.
        let opacity = configured_opacity
            .or_else(|| {
                configured_color
                    .as_deref()
                    .filter(|c| c.trim_start().starts_with("rgba("))
                    .map(super::color::alpha_percent)
            })
            .or_else(|| spec.setting.and_then(opacity_default_of))
            .unwrap_or(100);
        write_css_variable(css, spec.variable, super::color::compose(&resolved, opacity), "");
        return;
    }

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

    !spec.key.starts_with("widget-")
}

fn write_css_variable<T: std::fmt::Display>(css: &mut String, name: &str, value: T, unit: &str) {
    css.push_str(&format!("  {name}: {value}{unit};\n"));
}

#[cfg(test)]
mod golden {
    use super::CssVariables;
    use crate::config::StyleConfig;

    #[test]
    fn default_css_variables_match_golden() {
        let mut css = String::new();
        StyleConfig::default().write_css_variables(&mut css);

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden-css-variables.txt");
        if std::path::Path::new(path).exists() {
            let expected = std::fs::read_to_string(path).unwrap();
            assert_eq!(css, expected, "emitted CSS variables changed vs golden");
        } else {
            std::fs::write(path, &css).unwrap();
            panic!("wrote golden snapshot; re-run to assert");
        }
    }

    #[test]
    fn consumer_opacity_defaults_preserve_token_alpha() {
        use super::super::color::alpha_percent;
        // Pre-migration alpha-bearing palette token values.
        const OLD: &[(&str, &str)] = &[
            ("primary-container", "rgba(137, 180, 250, 0.22)"),
            ("secondary-container", "rgba(203, 166, 247, 0.18)"),
            ("tertiary-container", "rgba(253, 214, 100, 0.18)"),
            ("on-surface-variant", "rgba(241, 243, 244, 0.72)"),
            ("surface-container-lowest", "rgba(30, 30, 46, 0.96)"),
            ("surface-container-low", "rgba(241, 243, 244, 0.045)"),
            ("surface-container", "rgba(241, 243, 244, 0.06)"),
            ("surface-container-high", "rgba(241, 243, 244, 0.08)"),
            ("surface-container-highest", "rgba(241, 243, 244, 0.12)"),
            ("outline", "rgba(241, 243, 244, 0.14)"),
            ("outline-variant", "rgba(241, 243, 244, 0.08)"),
            ("error-container", "rgba(242, 139, 130, 0.18)"),
        ];
        let expected = |tok: &str| -> u16 {
            OLD.iter().find(|(t, _)| *t == tok).map(|(_, v)| alpha_percent(v)).unwrap_or(100)
        };
        for spec in super::specs::style_settings() {
            if spec.group == "palette" {
                continue;
            }
            if let Some(super::SettingUiSpec::Color { default: super::ColorDefault::Palette(tok), opacity_default, .. }) = spec.setting {
                assert_eq!(
                    opacity_default, expected(tok),
                    "{}/{} (-> {}) opacity_default {} != expected {}",
                    spec.group, spec.key, tok, opacity_default, expected(tok)
                );
            }
        }
    }

    #[test]
    fn consumer_colors_always_emit_composed_defaults() {
        let mut css = String::new();
        StyleConfig::default().write_css_variables(&mut css);

        // outline at 14% opacity - preserves old rgba(241,243,244,0.14)
        assert!(
            css.contains("  --osd-border-color: rgba(241, 243, 244, 0.140);"),
            "osd-border-color should emit composed default"
        );
        // primary-container at 22% opacity - preserves old rgba(137,180,250,0.22)
        assert!(
            css.contains("  --workspace-indicator-background-color: rgba(137, 180, 250, 0.220);"),
            "workspace-indicator-background-color should emit composed default"
        );
        // transparent literal stays transparent
        assert!(
            css.contains("  --bar-widget-background-color: transparent;"),
            "bar-widget-background-color should emit transparent"
        );
        // opaque palette token stays as solid hex
        assert!(
            css.contains("  --workspace-focused-color: #89b4fa;"),
            "workspace-focused-color should emit solid hex"
        );
    }
}
