use serde::Deserialize;
use std::{collections::BTreeMap, fs, io, path::PathBuf};

pub(crate) mod style;
pub(crate) mod variables;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BarRegionKey {
    Start,
    Center,
    End,
}

impl BarRegionKey {
    fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

pub(crate) fn set_bar_region(
    bar_name: &str,
    region: BarRegionKey,
    widgets: &[String],
) -> io::Result<()> {
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

    set_bar_region_in_document(&mut document, bar_name, region, widgets)?;

    fs::write(config_path, document.to_string())
}

fn set_bar_region_in_document(
    document: &mut toml_edit::DocumentMut,
    bar_name: &str,
    region: BarRegionKey,
    widgets: &[String],
) -> io::Result<()> {
    let Some(bars) = document["bars"].as_array_of_tables_mut() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "config does not contain [[bars]]",
        ));
    };

    let Some(bar) = bars
        .iter_mut()
        .find(|bar| bar.get("name").and_then(|item| item.as_str()) == Some(bar_name))
    else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("bar {bar_name} was not found"),
        ));
    };

    bar[region.as_str()] = toml_edit::value(string_array(widgets));

    Ok(())
}

pub(crate) fn set_bar_monitors(bar_name: &str, monitors: &[String]) -> io::Result<()> {
    let bar_name = bar_name.trim();

    if bar_name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bar name cannot be empty",
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

    set_bar_monitors_in_document(&mut document, bar_name, monitors)?;

    fs::write(config_path, document.to_string())
}

fn set_bar_monitors_in_document(
    document: &mut toml_edit::DocumentMut,
    bar_name: &str,
    monitors: &[String],
) -> io::Result<()> {
    let Some(bars) = document["bars"].as_array_of_tables_mut() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "config does not contain [[bars]]",
        ));
    };

    let Some(bar) = bars
        .iter_mut()
        .find(|bar| bar.get("name").and_then(|item| item.as_str()) == Some(bar_name))
    else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("bar {bar_name} was not found"),
        ));
    };

    if monitors.is_empty() {
        bar.remove("monitors");
    } else {
        bar["monitors"] = toml_edit::value(string_array(monitors));
    }

    Ok(())
}

pub(crate) fn add_bar(name: &str) -> io::Result<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bar name cannot be empty",
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

    add_bar_to_document(&mut document, name)?;

    fs::write(config_path, document.to_string())
}

fn add_bar_to_document(document: &mut toml_edit::DocumentMut, name: &str) -> io::Result<()> {
    if document["bars"].is_none() {
        document["bars"] = toml_edit::ArrayOfTables::new().into();
    }

    let Some(bars) = document["bars"].as_array_of_tables_mut() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "config bars must be an array of tables",
        ));
    };

    if bars
        .iter()
        .any(|bar| bar.get("name").and_then(|item| item.as_str()) == Some(name))
    {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("bar {name} already exists"),
        ));
    }

    let mut bar = toml_edit::Table::new();
    bar["name"] = toml_edit::value(name);
    bar["edge"] = toml_edit::value("top");
    bar["start"] = toml_edit::value(toml_edit::Array::new());
    bar["center"] = toml_edit::value(toml_edit::Array::new());
    bar["end"] = toml_edit::value(toml_edit::Array::new());

    bars.push(bar);

    Ok(())
}

pub(crate) fn remove_bar(name: &str) -> io::Result<()> {
    let name = name.trim();

    if name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bar name cannot be empty",
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

    remove_bar_from_document(&mut document, name)?;

    fs::write(config_path, document.to_string())
}

fn remove_bar_from_document(document: &mut toml_edit::DocumentMut, name: &str) -> io::Result<()> {
    let Some(bars) = document["bars"].as_array_of_tables_mut() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "config does not contain [[bars]]",
        ));
    };

    if bars.len() <= 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot remove the last bar",
        ));
    }

    let Some(index) = bars
        .iter()
        .position(|bar| bar.get("name").and_then(|item| item.as_str()) == Some(name))
    else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("bar {name} was not found"),
        ));
    };

    bars.remove(index);

    Ok(())
}

pub(crate) fn set_bar_edge(bar_name: &str, edge: &str) -> io::Result<()> {
    let bar_name = bar_name.trim();
    let edge = edge.trim();

    if bar_name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bar name cannot be empty",
        ));
    }

    if !is_valid_bar_edge(edge) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid bar edge: {edge}"),
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

    set_bar_edge_in_document(&mut document, bar_name, edge)?;

    fs::write(config_path, document.to_string())
}

fn set_bar_edge_in_document(
    document: &mut toml_edit::DocumentMut,
    bar_name: &str,
    edge: &str,
) -> io::Result<()> {
    let Some(bars) = document["bars"].as_array_of_tables_mut() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "config does not contain [[bars]]",
        ));
    };

    let Some(bar) = bars
        .iter_mut()
        .find(|bar| bar.get("name").and_then(|item| item.as_str()) == Some(bar_name))
    else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("bar {bar_name} was not found"),
        ));
    };

    bar["edge"] = toml_edit::value(edge);

    Ok(())
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

fn is_valid_bar_edge(edge: &str) -> bool {
    matches!(edge, "top" | "bottom" | "left" | "right")
}

fn string_array(values: &[String]) -> toml_edit::Array {
    let mut array = toml_edit::Array::new();

    for value in values {
        array.push(value.as_str());
    }

    array
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
