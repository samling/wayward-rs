use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct ActionMenuConfig {
    pub(super) panel: ActionMenuPanelConfig,
    pub(super) layout: ActionMenuLayoutConfig,
    pub(super) header: ActionMenuHeaderConfig,
    pub(super) sections: Vec<ActionMenuSectionConfig>,
}

impl Default for ActionMenuConfig {
    fn default() -> Self {
        Self {
            panel: ActionMenuPanelConfig::default(),
            layout: ActionMenuLayoutConfig::default(),
            header: ActionMenuHeaderConfig::default(),
            sections: vec![ActionMenuSectionConfig {
                title: Some("Screenshot".to_string()),
                columns: Some(3),
                actions: vec![
                    ActionMenuActionConfig::screenshot("Region", "\u{f125}", "region"),
                    ActionMenuActionConfig::screenshot("Window", "\u{f2d0}", "window"),
                    ActionMenuActionConfig::screenshot("Screen", "\u{f108}", "monitor"),
                ],
            }],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
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
            width: Some(268),
            max_height: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct ActionMenuHeaderConfig {
    pub(super) power_command: String,
    pub(super) power_args: Vec<String>,
}

impl Default for ActionMenuHeaderConfig {
    fn default() -> Self {
        Self {
            power_command: "wlogout".to_string(),
            power_args: Vec::new(),
        }
    }
}

// No deny_unknown_fields: tolerate deprecated keys (e.g. the removed `column-spacing`)
// in existing configs instead of failing the whole action_menu config parse.
#[derive(Clone, Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub(super) struct ActionMenuLayoutConfig {
    pub(super) columns: usize,
    pub(super) button_width: Option<i32>,
    pub(super) button_height: Option<i32>,
    pub(super) row_spacing: i32,
}

impl Default for ActionMenuLayoutConfig {
    fn default() -> Self {
        Self {
            columns: 3,
            button_width: Some(40),
            button_height: Some(40),
            row_spacing: 12,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct ActionMenuSectionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) columns: Option<usize>,
    pub(super) actions: Vec<ActionMenuActionConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct ActionMenuActionConfig {
    pub(super) label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) icon: Option<String>,
    #[serde(default)]
    pub(super) action: ActionMenuActionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) tooltip: Option<String>,
    #[serde(default = "default_show_label")]
    pub(super) show_label: bool,
}

fn default_show_label() -> bool {
    true
}

impl ActionMenuActionConfig {
    fn screenshot(label: &str, icon: &str, mode: &str) -> Self {
        let screenshot_path = dirs::home_dir()
            .map(|home| home.join(".local/bin/screenshot"))
            .unwrap_or_else(|| std::path::PathBuf::from(".local/bin/screenshot"));

        Self {
            label: label.to_string(),
            icon: Some(icon.to_string()),
            action: ActionMenuActionKind::Command,
            command: Some(screenshot_path.to_string_lossy().to_string()),
            args: vec![mode.to_string()],
            class: Some("action-menu-screenshot-action".to_string()),
            tooltip: Some(format!("Screenshot {}", label.to_lowercase())),
            show_label: true,
        }
    }
}
