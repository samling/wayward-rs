use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct ActionMenuConfig {
    pub(super) panel: ActionMenuPanelConfig,
    pub(super) layout: ActionMenuLayoutConfig,
    pub(super) sections: Vec<ActionMenuSectionConfig>,
}

impl Default for ActionMenuConfig {
    fn default() -> Self {
        Self {
            panel: ActionMenuPanelConfig::default(),
            layout: ActionMenuLayoutConfig::default(),
            sections: vec![
                ActionMenuSectionConfig {
                    title: None,
                    columns: Some(2),
                    align: ActionMenuSectionAlign::End,
                    actions: vec![
                        ActionMenuActionConfig {
                            label: "Settings".to_string(),
                            icon: Some("\u{f013}".to_string()),
                            action: ActionMenuActionKind::OpenSettings,
                            command: None,
                            args: Vec::new(),
                            class: Some("action-menu-settings".to_string()),
                            tooltip: Some("Settings".to_string()),
                            show_label: false,
                        },
                        ActionMenuActionConfig {
                            label: "Power menu".to_string(),
                            icon: Some("\u{f011}".to_string()),
                            action: ActionMenuActionKind::Command,
                            command: Some("wlogout".to_string()),
                            args: Vec::new(),
                            class: Some("action-menu-power".to_string()),
                            tooltip: Some("Power menu".to_string()),
                            show_label: false,
                        }
                    ],
                },
                ActionMenuSectionConfig {
                    title: Some("Screenshot".to_string()),
                    columns: Some(3),
                    align: ActionMenuSectionAlign::Fill,
                    actions: vec![
                        ActionMenuActionConfig::screenshot("Region", "\u{f125}", "region"),
                        ActionMenuActionConfig::screenshot("Window", "\u{f2d0}", "window"),
                        ActionMenuActionConfig::screenshot("Screen", "\u{f108}", "monitor"),
                    ],
                },
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ActionMenuSectionAlign {
    Start,
    Center,
    End,
    Fill,
}

impl Default for ActionMenuSectionAlign {
    fn default() -> Self {
        Self::Fill
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ActionMenuActionKind {
    #[default]
    Command,
    OpenSettings,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct ActionMenuPanelConfig {
    pub(super) width: Option<i32>,
    pub(super) max_height: Option<i32>,
}

impl Default for ActionMenuPanelConfig {
    fn default() -> Self {
        Self {
            width: None,
            max_height: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct ActionMenuLayoutConfig {
    pub(super) columns: usize,
    pub(super) button_width: Option<i32>,
    pub(super) button_height: Option<i32>,
    pub(super) row_spacing: i32,
    pub(super) column_spacing: i32,
}

impl Default for ActionMenuLayoutConfig {
    fn default() -> Self {
        Self {
            columns: 3,
            button_width: None,
            button_height: None,
            row_spacing: 8,
            column_spacing: 8,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct ActionMenuSectionConfig {
    pub(super) title: Option<String>,
    pub(super) columns: Option<usize>,
    pub(super) actions: Vec<ActionMenuActionConfig>,
    pub(super) align: ActionMenuSectionAlign,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct ActionMenuActionConfig {
    pub(super) label: String,
    #[serde(default)]
    pub(super) icon: Option<String>,
    #[serde(default)]
    pub(super) action: ActionMenuActionKind,
    #[serde(default)]
    pub(super) command: Option<String>,
    #[serde(default)]
    pub(super) args: Vec<String>,
    #[serde(default)]
    pub(super) class: Option<String>,
    #[serde(default)]
    pub(super) tooltip: Option<String>,
    #[serde(default = "default_show_label")]
    pub(super) show_label: bool,
}

fn default_show_label() -> bool {
    true
}

impl ActionMenuActionConfig {
    fn screenshot(label: &str, icon: &str, mode: &str) -> Self {
        Self {
            label: label.to_string(),
            icon: Some(icon.to_string()),
            action: ActionMenuActionKind::Command,
            command: Some("screenshot".to_string()),
            args: vec![mode.to_string()],
            class: Some("action-menu-screenshot-action".to_string()),
            tooltip: Some(format!("Screenshot {}", label.to_lowercase())),
            show_label: true,
        }
    }
}