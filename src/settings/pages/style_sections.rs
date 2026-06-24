use crate::config::{
    StyleConfig,
    style::StyleGroupExt,
    variables::{SettingUiSpec, settings_for_section},
};

use super::super::spec::{
    ColorSettingRole, ColorSpec, NumberSpec, PaletteOption, SettingSpec, SettingsSectionSpec,
    StringSpec, ToggleSpec,
};

pub(crate) fn palette_options(style: &StyleConfig) -> Vec<PaletteOption> {
    use crate::config::variables::settings_for_section;
    settings_for_section("Palette")
        .filter_map(|spec| {
            if spec.group != "palette" {
                return None;
            }
            let resolved = style
                .group("palette")
                .and_then(|g| g.string(spec.key))
                .or_else(|| crate::config::variables::palette_color_default(spec.key).map(str::to_string))?;
            let hex = crate::config::color::solid_hex(&resolved).unwrap_or(resolved);
            Some(PaletteOption {
                token: spec.key,
                label: spec.key.replace('-', " "),
                hex,
            })
        })
        .collect()
}

fn opacity_path_for(spec: &crate::config::variables::StyleSettingSpec) -> Vec<String> {
    let mut path: Vec<String> = spec.path.iter().map(|p| p.to_string()).collect();
    if let Some(last) = path.last_mut() {
        *last = crate::config::variables::opacity_key(spec.key);
    }
    path
}

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
                SettingUiSpec::Color { label, default, opacity_default } => {
                    let raw = group.and_then(|group| group.string(spec.key));
                    let is_palette_ref = match raw.as_deref() {
                        Some(v) => crate::config::color::parse_rgb(v).is_none() && v != "transparent",
                        // No stored value - palette default counts as a palette reference.
                        None => matches!(default, crate::config::variables::ColorDefault::Palette(_)),
                    };
                    let opacity = group.and_then(|g| {
                        g.integer(&crate::config::variables::opacity_key(spec.key))
                    });
                    Some(SettingSpec::Color(ColorSpec {
                        label,
                        path: spec.path,
                        value: raw,
                        default: default.resolve(style),
                        inherited: inherited_color(spec, style, default),
                        role: color_setting_role(spec),
                        opacity,
                        opacity_default,
                        opacity_path: opacity_path_for(spec),
                        is_palette_ref,
                        palette_options: palette_options(style),
                    }))
                }
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

fn inherited_color(
    spec: &crate::config::variables::StyleSettingSpec,
    style: &StyleConfig,
    default: crate::config::variables::ColorDefault,
) -> Option<String> {
    if color_setting_role(spec) != ColorSettingRole::Override {
        return None;
    }

    if spec.group != "bar" && matches!(spec.key, "widget-background-color" | "widget-border-color")
    {
        return Some(
            style
                .group("bar")
                .and_then(|group| group.string(spec.key))
                .unwrap_or_else(|| default.resolve(style)),
        );
    }

    Some(default.resolve(style))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StyleConfig;
    use super::super::super::spec::SettingSpec;

    #[test]
    fn palette_options_are_solid_and_nonempty() {
        let options = super::palette_options(&StyleConfig::default());
        assert!(!options.is_empty());
        assert!(options.iter().any(|o| o.token == "primary"));
        for option in &options {
            assert!(option.hex.starts_with('#'), "{} not solid: {}", option.token, option.hex);
        }
    }

    #[test]
    fn consumer_color_spec_carries_opacity_and_palette_ref() {
        let section = section("Bar", &StyleConfig::default());
        let spec = section
            .settings
            .iter()
            .find_map(|s| match s {
                SettingSpec::Color(c) if c.path == ["style", "bar", "widget-border-color"] => Some(c),
                _ => None,
            })
            .expect("widget-border-color color spec");
        assert_eq!(spec.opacity_default, 8);
        assert!(spec.is_palette_ref);
        assert_eq!(spec.opacity_path, vec!["style", "bar", "widget-border-opacity"]);
    }
}
