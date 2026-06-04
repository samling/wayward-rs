use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::PathBuf};

const DEFAULT_CONFIG_TOML: &str = r#"# Wayward app configuration.
# theme = "example"

# [notifications]
# monitor = "DP-1"

[[bars]]
name = "bar"
edge = "top"
start = ["workspaces"]
center = ["clock"]
end = ["systray"]
"#;

pub(crate) fn themes_dir() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("themes"))
}

pub(crate) fn ensure_config_files() {
    let Some(dir) = config_dir() else {
        tracing::info!("Could not determine config direcotry, skipping config bootstrap");
        return;
    };

    if let Err(error) = fs::create_dir_all(&dir) {
        tracing::error!(
            "Failed to create config directory {}: {error}",
            dir.display()
        );
        return;
    }

    if let Some(themes_dir) = themes_dir() {
        if let Err(error) = fs::create_dir_all(&themes_dir) {
            tracing::error!(
                "Failed to create themes directory {}: {error}",
                themes_dir.display()
            )
        }
    }

    write_default_file(config_path(), DEFAULT_CONFIG_TOML);
}

fn write_default_file(path: Option<PathBuf>, contents: &str) {
    let Some(path) = path else {
        return;
    };

    if path.exists() {
        return;
    }

    if let Err(error) = fs::write(&path, &contents) {
        tracing::error!("Failed to create default file {}: {error}", path.display())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub widgets: BTreeMap<String, toml::value::Table>,
    #[serde(default)]
    pub notifications: NotificationConfig,
    pub bars: Vec<BarConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationConfig {
    #[serde(default)]
    pub monitor: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BarConfig {
    pub name: Option<String>,
    pub edge: Option<String>,
    pub monitors: Option<Vec<String>>,
    pub start: Option<Vec<String>>,
    pub center: Option<Vec<String>>,
    pub end: Option<Vec<String>>,
}

impl AppConfig {
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            tracing::info!("Could not determine config directory, using defaults");
            return Self::default();
        };

        let Ok(contents) = fs::read_to_string(&path) else {
            tracing::info!("No config file found at {}, using defaults", path.display());
            return Self::default();
        };

        match toml::from_str(&contents) {
            Ok(config) => config,
            Err(error) => {
                tracing::error!("Failed to parse config at {}: {error}", path.display());
                Self::default()
            }
        }
    }
}

pub(crate) fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("wayward"))
}

pub(crate) fn config_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("config.toml"))
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: None,
            widgets: BTreeMap::new(),
            notifications: NotificationConfig::default(),
            bars: vec![BarConfig::default()],
        }
    }
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            name: None,
            edge: None,
            monitors: None,
            start: None,
            center: None,
            end: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_accepts_notification_monitor() {
        let config: AppConfig = toml::from_str(
            r#"
[notifications]
monitor = "DP-1"

[[bars]]
start = []
center = []
end = []
"#,
        )
        .unwrap();

        assert_eq!(config.notifications.monitor.as_deref(), Some("DP-1"));
    }

    #[test]
    fn config_defaults_notification_monitor_to_focused_monitor() {
        let config: AppConfig = toml::from_str(
            r#"
[[bars]]
start = []
center = []
end = []
"#,
        )
        .unwrap();

        assert_eq!(config.notifications.monitor, None);
    }
}
