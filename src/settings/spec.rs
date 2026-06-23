use crate::config::ConfigValue;

#[derive(Clone, Debug)]
pub(crate) struct SettingsPageSpec {
    pub(crate) title: String,
    pub(crate) sections: Vec<SettingsSectionSpec>,
}

#[derive(Clone, Debug)]
pub(crate) struct SettingsSectionSpec {
    pub(crate) title: String,
    pub(crate) settings: Vec<SettingSpec>,
}

#[derive(Clone, Debug)]
pub(crate) enum SettingSpec {
    Number(NumberSpec),
    Toggle(ToggleSpec),
    String(StringSpec),
    StringList(StringListSpec),
    Color(ColorSpec),
}

#[derive(Clone, Debug)]
pub(crate) struct NumberSpec {
    pub(crate) label: &'static str,
    pub(crate) path: &'static [&'static str],
    pub(crate) value: Option<u16>,
    pub(crate) default: u16,
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) step: f64,
}

impl NumberSpec {
    pub(crate) fn display_value(&self) -> f64 {
        self.value.unwrap_or(self.default) as f64
    }

    pub(crate) fn value_for_config(&self, value: f64) -> ConfigValue {
        ConfigValue::Integer(value as i64)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ToggleSpec {
    pub(crate) label: &'static str,
    pub(crate) path: &'static [&'static str],
    pub(crate) value: Option<bool>,
    pub(crate) default: bool,
}

impl ToggleSpec {
    pub(crate) fn display_value(&self) -> bool {
        self.value.unwrap_or(self.default)
    }

    pub(crate) fn value_for_config(&self, value: bool) -> ConfigValue {
        ConfigValue::Bool(value)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StringSpec {
    pub(crate) label: &'static str,
    pub(crate) path: &'static [&'static str],
    pub(crate) value: Option<String>,
    pub(crate) default: &'static str,
}

impl StringSpec {
    pub(crate) fn display_value(&self) -> String {
        self.value
            .clone()
            .unwrap_or_else(|| self.default.to_string())
    }

    pub(crate) fn value_for_config(&self, value: String) -> ConfigValue {
        ConfigValue::String(value)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StringListSpec {
    pub(crate) label: &'static str,
    pub(crate) path: &'static [&'static str],
    pub(crate) value: Option<Vec<String>>,
    pub(crate) default: &'static [&'static str],
}

impl StringListSpec {
    pub(crate) fn display_value(&self) -> String {
        self.value
            .clone()
            .unwrap_or_else(|| self.default.iter().map(|value| value.to_string()).collect())
            .join(", ")
    }

    pub(crate) fn value_for_config(&self, value: String) -> ConfigValue {
        ConfigValue::StringList(parse_string_list(&value))
    }
}

fn parse_string_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[derive(Clone, Debug)]
pub(crate) struct ColorSpec {
    pub(crate) label: &'static str,
    pub(crate) path: &'static [&'static str],
    pub(crate) value: Option<String>,
    pub(crate) default: &'static str,
    pub(crate) inherited: Option<String>,
    pub(crate) role: ColorSettingRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ColorSettingRole {
    Palette,
    Default,
    Override,
}

impl ColorSpec {
    pub(crate) fn display_value(&self) -> String {
        self.value
            .clone()
            .or_else(|| self.inherited.clone())
            .unwrap_or_else(|| self.default.to_string())
    }

    pub(crate) fn entry_value(&self) -> String {
        if self.is_inherited() {
            String::new()
        } else {
            self.display_value()
        }
    }

    pub(crate) fn display_label(&self) -> String {
        self.label.to_string()
    }

    pub(crate) fn placeholder(&self) -> Option<&'static str> {
        self.is_inherited().then_some("Inherited")
    }

    pub(crate) fn is_inherited(&self) -> bool {
        self.role == ColorSettingRole::Override && self.value.is_none()
    }

    pub(crate) fn is_custom(&self) -> bool {
        self.value.is_some()
    }

    pub(crate) fn reset_tooltip(&self) -> &'static str {
        match self.role {
            ColorSettingRole::Palette => "Reset palette color",
            ColorSettingRole::Default => "Reset default color",
            ColorSettingRole::Override => "Remove override",
        }
    }

    pub(crate) fn value_for_config(&self, value: String) -> ConfigValue {
        ConfigValue::String(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_list_spec_displays_values_as_comma_separated_text() {
        let setting = StringListSpec {
            label: "Critical patterns",
            path: &["widgets", "updates", "critical-patterns"],
            value: Some(vec!["linux-*".to_string(), "pacman".to_string()]),
            default: &[],
        };

        assert_eq!(setting.display_value(), "linux-*, pacman");
    }

    #[test]
    fn string_list_spec_parses_comma_separated_text_for_config() {
        let setting = StringListSpec {
            label: "Critical patterns",
            path: &["widgets", "updates", "critical-patterns"],
            value: None,
            default: &[],
        };

        assert_eq!(
            setting.value_for_config(" linux-* , pacman,, ".to_string()),
            ConfigValue::StringList(vec!["linux-*".to_string(), "pacman".to_string()])
        );
    }
}
