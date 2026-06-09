use super::style::{StyleConfig, StyleGroupConfig, StyleGroupExt};

const CSS_VARIABLES: &[CssVariableSpec] = &[
    CssVariableSpec {
        group: "notifications",
        key: "body-font-weight",
        variable: "--notification-body-font-weight",
        kind: CssValueKind::Integer { unit: "" },
    },
    CssVariableSpec {
        group: "notifications",
        key: "normal-border-width",
        variable: "--notification-normal-border-width",
        kind: CssValueKind::Integer { unit: "px" },
    },
    CssVariableSpec {
        group: "notifications",
        key: "list-icon-size",
        variable: "--notification-list-icon-size",
        kind: CssValueKind::Integer { unit: "px" },
    },
    CssVariableSpec {
        group: "notifications",
        key: "hide-scrollbar",
        variable: "--notification-scrollbar-opacity",
        kind: CssValueKind::Bool {
            true_value: "0",
            false_value: "1",
        },
    },
    CssVariableSpec {
        group: "notifications",
        key: "font-family",
        variable: "--notification-font-family",
        kind: CssValueKind::String { quoted: true },
    },
];

pub(crate) trait CssVariables {
    fn write_css_variables(&self, css: &mut String);
}

#[derive(Clone, Copy)]
enum CssValueKind {
    Integer { unit: &'static str },
    String { quoted: bool },
    Bool {
        true_value: &'static str,
        false_value: &'static str,
    },
}

struct CssVariableSpec {
    group: &'static str,
    key: &'static str,
    variable: &'static str,
    kind: CssValueKind,
}

impl CssVariables for StyleConfig {
    fn write_css_variables(&self, css: &mut String) {
        for spec in CSS_VARIABLES {
            let Some(group) = self.group(spec.group) else {
                continue;
            };

            write_mapped_css_variable(css, group, spec);
        }
    }
}

fn write_mapped_css_variable(
    css: &mut String,
    group: &StyleGroupConfig,
    spec: &CssVariableSpec,
) {
    match spec.kind {
        CssValueKind::Integer { unit } => {
            if let Some(value) = group.integer(spec.key) {
                write_css_variable(css, spec.variable, value, unit);
            }
        }
        CssValueKind::String { quoted } => {
            if let Some(value) = group.string(spec.key) {
                let value = if quoted { format!("\"{value}\"") } else { value };
                write_css_variable(css, spec.variable, value, "");
            }
        }
        CssValueKind::Bool { true_value, false_value } => {
            if let Some(value) = group.bool(spec.key) {
                let value = if value { true_value } else { false_value };
                write_css_variable(css, spec.variable, value, "");
            }
        }
    }
}

fn write_css_variable<T: std::fmt::Display>(css: &mut String, name: &str, value: T, unit: &str) {
    css.push_str(&format!("  {name}: {value}{unit};\n"));
}
