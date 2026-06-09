use super::ConfigValue;
use serde::Deserialize;
use std::collections::BTreeMap;

pub(crate) type StyleGroupConfig = BTreeMap<String, StyleValue>;

const BAR_GROUP: &str = "bar";
const NOTIFICATIONS_GROUP: &str = "notifications";

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StyleConfig {
    #[serde(default)]
    pub bar: StyleGroupConfig,
    #[serde(default)]
    pub notifications: StyleGroupConfig,
}

impl StyleConfig {
    pub(crate) fn group(&self, group: &str) -> Option<&StyleGroupConfig> {
        match group {
            BAR_GROUP => Some(&self.bar),
            NOTIFICATIONS_GROUP => Some(&self.notifications),
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
            NOTIFICATIONS_GROUP => Some(&mut self.notifications),
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
