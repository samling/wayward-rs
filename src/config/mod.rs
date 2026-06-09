use serde::Deserialize;
use std::{collections::BTreeMap, fs, io, path::PathBuf};

pub(crate) mod style;
pub(crate) use style::StyleConfig;

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

#[derive(Clone, Debug)]
pub(crate) enum ConfigValue {
    Integer(i64),
    String(String),
    Bool(bool),
}

impl ConfigValue {
    fn into_item(self) -> toml_edit::Item {
        match self {
            Self::Bool(value) => toml_edit::value(value),
            Self::Integer(value) => toml_edit::value(value),
            Self::String(value) => toml_edit::value(value),
        }
    }
}

pub(crate) fn set_config_value(path: &[&str], value: Option<ConfigValue>) -> io::Result<()> {
    if path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "config path cannot be empty",
        ));
    }

    let Some(config_path) = config_path() else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "could not determine config path",
        ));
    };

    let contents = fs::read_to_string(&config_path).unwrap_or_default();
    let mut document = contents
        .parse::<toml_edit::DocumentMut>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    set_document_value(&mut document, path, value);

    fs::write(config_path, document.to_string())
}

fn set_document_value(
    document: &mut toml_edit::DocumentMut,
    path: &[&str],
    value: Option<ConfigValue>,
) {
    let mut item = document.as_item_mut();

    for segment in &path[..path.len() - 1] {
        item[*segment].or_insert(toml_edit::table());
        item = &mut item[*segment];
    }

    let key = path[path.len() - 1];

    if let Some(value) = value {
        item[key] = value.into_item();
    } else if let Some(table) = item.as_table_like_mut() {
        table.remove(key);
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub widgets: BTreeMap<String, toml::value::Table>,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub style: StyleConfig,
    pub bars: Vec<BarConfig>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NotificationConfig {
    #[serde(default)]
    pub monitor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BarConfig {
    pub name: Option<String>,
    pub edge: Option<String>,
    pub monitors: Option<Vec<String>>,
    pub start: Option<Vec<String>>,
    pub center: Option<Vec<String>>,
    pub end: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ConfigChanges {
    pub(crate) bars_changed: bool,
    pub(crate) notifications_changed: bool,
    pub(crate) style_changed: bool,
    pub(crate) widgets_changed: bool,
}

impl ConfigChanges {
    pub(crate) fn between(previous: &AppConfig, next: &AppConfig) -> Self {
        Self {
            bars_changed: previous.bars != next.bars,
            notifications_changed: previous.notifications != next.notifications,
            style_changed: previous.theme != next.theme || previous.style != next.style,
            widgets_changed: previous.widgets != next.widgets,
        }
    }

    pub(crate) fn has_changes(&self) -> bool {
        self.bars_changed
            || self.notifications_changed
            || self.style_changed
            || self.widgets_changed
    }
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
            style: StyleConfig::default(),
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

    fn config_with_notification_monitor(monitor: Option<&str>) -> AppConfig {
        AppConfig {
            notifications: NotificationConfig {
                monitor: monitor.map(ToOwned::to_owned),
            },
            ..AppConfig::default()
        }
    }

    fn config_with_theme(theme: Option<&str>) -> AppConfig {
        AppConfig {
            theme: theme.map(ToOwned::to_owned),
            ..AppConfig::default()
        }
    }

    #[test]
    fn config_changes_detects_noop_reload() {
        let previous = AppConfig::default();
        let next = AppConfig::default();

        assert_eq!(
            ConfigChanges::between(&previous, &next),
            ConfigChanges::default()
        );
        assert!(!ConfigChanges::between(&previous, &next).has_changes());
    }

    #[test]
    fn config_changes_detects_notification_domain() {
        let previous = config_with_notification_monitor(None);
        let next = config_with_notification_monitor(Some("DP-1"));

        let changes = ConfigChanges::between(&previous, &next);

        assert!(changes.notifications_changed);
        assert!(!changes.bars_changed);
        assert!(!changes.style_changed);
        assert!(!changes.widgets_changed);
    }

    #[test]
    fn config_changes_detects_style_domain() {
        let previous = config_with_theme(None);
        let next = config_with_theme(Some("dark"));

        let changes = ConfigChanges::between(&previous, &next);

        assert!(changes.style_changed);
        assert!(!changes.bars_changed);
        assert!(!changes.notifications_changed);
        assert!(!changes.widgets_changed);
    }

    #[test]
    fn config_accepts_notification_style_controls() {
        let config: AppConfig = toml::from_str(
            r#"
    [style.notifications]
    body-font-weight = 500
    normal-border-width = 2

    [[bars]]
    name = "bar"
    start = []
    center = []
    end = []
    "#,
        )
        .unwrap();

        use crate::config::style::StyleGroupExt;

        assert_eq!(
            config.style.notifications.integer("body-font-weight"),
            Some(500)
        );
        assert_eq!(
            config.style.notifications.integer("normal-border-width"),
            Some(2)
        );
    }
}
