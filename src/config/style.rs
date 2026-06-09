use serde::Deserialize;

pub(crate) trait CssVariables {
    fn write_css_variables(&self, css: &mut String);
}

impl CssVariables for StyleConfig {
    fn write_css_variables(&self, css: &mut String) {
        self.notifications.write_css_variables(css);
    }
}

impl CssVariables for NotificationStyleConfig {
    fn write_css_variables(&self, css: &mut String) {
        write_optional(css, "--notification-body-font-weight", self.body_font_weight, "");
        write_optional(
            css,
            "--notification-normal-border-width",
            self.normal_border_width_px,
            "px",
        );
        
        if let Some(hide_scrollbar) = self.hide_scrollbar {
            let opacity = if hide_scrollbar { 0 } else { 1 };
            write_css_variable(css, "--notification-scrollbar-opacity", opacity, "");
        }

        if let Some(font_family) = &self.font_family {
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

fn write_css_variable<T: std::fmt::Display>(
    css: &mut String,
    name: &str,
    value: T,
    unit: &str,
) {
    css.push_str(&format!("  {name}: {value}{unit};\n"));
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StyleConfig {
    #[serde(default)]
    pub notifications: NotificationStyleConfig,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NotificationStyleConfig {
    #[serde(default)]
    pub body_font_weight: Option<u16>,
    #[serde(default)]
    pub normal_border_width_px: Option<u16>,
    #[serde(default)]
    pub hide_scrollbar: Option<bool>,
    #[serde(default)]
    pub font_family: Option<String>,
}
