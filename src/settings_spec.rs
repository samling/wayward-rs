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
    Choice(ChoiceSpec),
}

#[derive(Clone, Debug)]
pub(crate) struct NumberSpec {
    pub(crate) label: &'static str,
    pub(crate) description: Option<&'static str>,
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
    pub(crate) description: Option<&'static str>,
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

#[derive(Clone, Copy, Debug)]
pub(crate) struct ChoiceOption {
    pub(crate) value: &'static str,
    pub(crate) label: &'static str,
}

#[derive(Clone, Debug)]
pub(crate) struct ChoiceSpec {
    pub(crate) label: &'static str,
    pub(crate) description: Option<&'static str>,
    pub(crate) path: &'static [&'static str],
    pub(crate) value: Option<String>,
    pub(crate) default: &'static str,
    pub(crate) options: &'static [ChoiceOption],
}

impl ChoiceSpec {
    pub(crate) fn display_value(&self) -> String {
        self.value
            .clone()
            .unwrap_or_else(|| self.default.to_string())
    }

    pub(crate) fn selected_index(&self) -> u32 {
        let current = self.display_value();
        self.options
            .iter()
            .position(|option| option.value == current)
            .unwrap_or(0) as u32
    }

    pub(crate) fn value_for_config(&self, index: u32) -> ConfigValue {
        let value = self
            .options
            .get(index as usize)
            .map(|option| option.value)
            .unwrap_or(self.default);
        ConfigValue::String(value.to_string())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StringSpec {
    pub(crate) label: &'static str,
    pub(crate) description: Option<&'static str>,
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
    pub(crate) description: Option<&'static str>,
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

pub(crate) fn table_string(table: &toml::value::Table, path: &[&str]) -> Option<String> {
    let mut current = table;
    for key in &path[..path.len().saturating_sub(1)] {
        current = current.get(*key)?.as_table()?;
    }
    current.get(*path.last()?)?.as_str().map(ToOwned::to_owned)
}

pub(crate) fn table_string_list(table: &toml::value::Table, key: &str) -> Option<Vec<String>> {
    let values = table.get(key)?.as_array()?;
    Some(
        values
            .iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect(),
    )
}

pub(crate) fn table_u16(table: &toml::value::Table, key: &str) -> Option<u16> {
    table.get(key)?.as_integer()?.try_into().ok()
}

#[derive(Clone, Debug)]
pub(crate) struct PaletteOption {
    pub(crate) token: &'static str,
    pub(crate) label: String,
    pub(crate) color: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ColorSpec {
    pub(crate) label: &'static str,
    pub(crate) path: &'static [&'static str],
    pub(crate) value: Option<String>,
    pub(crate) default: String,
    pub(crate) default_token: Option<&'static str>,
    pub(crate) inherited: Option<String>,
    pub(crate) role: ColorSettingRole,
    pub(crate) opacity: Option<u16>,
    pub(crate) opacity_default: u16,
    pub(crate) opacity_path: Vec<String>,
    pub(crate) is_palette_ref: bool,
    pub(crate) palette_options: Vec<PaletteOption>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ColorSettingRole {
    Palette,
    Default,
    Override,
}

impl ColorSpec {
    pub(crate) fn display_opacity(&self) -> u16 {
        self.opacity.unwrap_or(self.opacity_default)
    }

    pub(crate) fn display_value(&self) -> String {
        self.value
            .clone()
            .or_else(|| self.inherited.clone())
            .unwrap_or_else(|| self.default.clone())
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
#[path = "settings_spec_test.rs"]
mod tests;
