use serde::Deserialize;
use std::collections::BTreeMap;

pub(crate) trait CssVariables {
    fn write_css_variables(&self, css: &mut String);
}

pub(crate) type StyleGroupConfig = BTreeMap<String, StyleValue>;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StyleConfig {
    #[serde(default)]
    pub notifications: StyleGroupConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum StyleValue {
    Integer(i64),
    Bool(bool),
    String(String),
}

pub(crate) trait StyleGroupExt {
    fn integer(&self, key: &str) -> Option<u16>;
    fn bool(&self, key: &str) -> Option<bool>;
    fn string(&self, key: &str) -> Option<String>;
}

impl StyleGroupExt for StyleGroupConfig {
    fn integer(&self, key: &str) -> Option<u16> {
        match self.get(key) {
            Some(StyleValue::Integer(value)) => (*value).try_into().ok(),
            _ => None,
        }
    }

    fn bool(&self, key: &str) -> Option<bool> {
        match self.get(key) {
            Some(StyleValue::Bool(value)) => Some(*value),
            _ => None,
        }
    }

    fn string(&self, key: &str) -> Option<String> {
        match self.get(key) {
            Some(StyleValue::String(value)) => Some(value.clone()),
            _ => None,
        }
    }
}

impl CssVariables for StyleConfig {
    fn write_css_variables(&self, css: &mut String) {
        self.notifications.write_css_variables(css);
    }
}

impl CssVariables for StyleGroupConfig {
    fn write_css_variables(&self, css: &mut String) {
        write_optional(
            css,
            "--notification-body-font-weight",
            self.integer("body-font-weight"),
            "",
        );
        write_optional(
            css,
            "--notification-normal-border-width",
            self.integer("normal-border-width"),
            "px",
        );

        if let Some(hide_scrollbar) = self.bool("hide-scrollbar") {
            let opacity = if hide_scrollbar { 0 } else { 1 };
            write_css_variable(css, "--notification-scrollbar-opacity", opacity, "");
        }

        if let Some(font_family) = &self.string("font-family") {
            write_css_variable(
                css,
                "--notification-font-family",
                format!("\"{font_family}\""),
                "",
            );
        }
    }
}

fn write_optional<T: std::fmt::Display>(
    css: &mut String,
    name: &str,
    value: Option<T>,
    unit: &str,
) {
    if let Some(value) = value {
        write_css_variable(css, name, value, unit);
    }
}

fn write_css_variable<T: std::fmt::Display>(css: &mut String, name: &str, value: T, unit: &str) {
    css.push_str(&format!("  {name}: {value}{unit};\n"));
}
