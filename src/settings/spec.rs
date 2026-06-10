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