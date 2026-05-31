use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub bars: Vec<BarConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BarConfig {
    pub name: Option<String>,
    pub edge: Option<String>,
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
            bars: vec![BarConfig::default()],
        }
    }
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            name: None,
            edge: None,
            start: None,
            center: None,
            end: None,
        }
    }
}
