use super::ConfigValue;
use serde::Deserialize;
use std::collections::BTreeMap;

pub(crate) type StyleGroupConfig = BTreeMap<String, StyleValue>;

const BAR_GROUP: &str = "bar";
const WORKSPACES_GROUP: &str = "workspaces";
const SURFACES_GROUP: &str = "surfaces";
const DROPDOWNS_GROUP: &str = "dropdowns";
const BATTERY_GROUP: &str = "battery";
const SYSTRAY_GROUP: &str = "systray";
const OSD_GROUP: &str = "osd";
const NOTIFICATIONS_GROUP: &str = "notifications";
const ACTION_MENU_GROUP: &str = "action-menu";
const SETTINGS_GROUP: &str = "settings";

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StyleConfig {
    #[serde(default)]
    pub bar: StyleGroupConfig,
    #[serde(default)]
    pub workspaces: StyleGroupConfig,
    #[serde(default)]
    pub surfaces: StyleGroupConfig,
    #[serde(default)]
    pub dropdowns: StyleGroupConfig,
    #[serde(default)]
    pub battery: StyleGroupConfig,
    #[serde(default)]
    pub systray: StyleGroupConfig,
    #[serde(default)]
    pub osd: StyleGroupConfig,
    #[serde(default)]
    pub notifications: StyleGroupConfig,
    #[serde(default, rename = "action-menu")]
    pub action_menu: StyleGroupConfig,
    #[serde(default)]
    pub settings: StyleGroupConfig,
}

impl StyleConfig {
    pub(crate) fn group(&self, group: &str) -> Option<&StyleGroupConfig> {
        match group {
            BAR_GROUP => Some(&self.bar),
            WORKSPACES_GROUP => Some(&self.workspaces),
            SURFACES_GROUP => Some(&self.surfaces),
            DROPDOWNS_GROUP => Some(&self.dropdowns),
            BATTERY_GROUP => Some(&self.battery),
            SYSTRAY_GROUP => Some(&self.systray),
            OSD_GROUP => Some(&self.osd),
            NOTIFICATIONS_GROUP => Some(&self.notifications),
            ACTION_MENU_GROUP => Some(&self.action_menu),
            SETTINGS_GROUP => Some(&self.settings),
            _ => None,
        }
    }

    pub(crate) fn apply_config_value(
        &mut self,
        path: &[&str],
        value: Option<&ConfigValue>,
    ) -> bool {
        let ["style", group, key] = path else {
            return false;
        };

        let Some(group) = self.group_mut(group) else {
            return false;
        };

        match value {
            Some(value) => {
                group.insert((*key).to_string(), StyleValue::from(value));
            }
            None => {
                group.remove(*key);
            }
        }

        true
    }

    fn group_mut(&mut self, group: &str) -> Option<&mut StyleGroupConfig> {
        match group {
            BAR_GROUP => Some(&mut self.bar),
            WORKSPACES_GROUP => Some(&mut self.workspaces),
            SURFACES_GROUP => Some(&mut self.surfaces),
            DROPDOWNS_GROUP => Some(&mut self.dropdowns),
            BATTERY_GROUP => Some(&mut self.battery),
            SYSTRAY_GROUP => Some(&mut self.systray),
            OSD_GROUP => Some(&mut self.osd),
            NOTIFICATIONS_GROUP => Some(&mut self.notifications),
            ACTION_MENU_GROUP => Some(&mut self.action_menu),
            SETTINGS_GROUP => Some(&mut self.settings),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum StyleValue {
    Integer(i64),
    Bool(bool),
    String(String),
}

impl From<&ConfigValue> for StyleValue {
    fn from(value: &ConfigValue) -> Self {
        match value {
            ConfigValue::Integer(value) => Self::Integer(*value),
            ConfigValue::Bool(value) => Self::Bool(*value),
            ConfigValue::String(value) => Self::String(value.clone()),
        }
    }
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
