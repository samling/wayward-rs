use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct BrightnessConfig {
    pub(super) blue_light_enable_command: String,
    pub(super) blue_light_disable_command: String,
    pub(super) blue_light_state_command: String,
}

impl BrightnessConfig {
    pub(super) fn blue_light_toggle_configured(&self) -> bool {
        !self.blue_light_enable_command.trim().is_empty()
            && !self.blue_light_disable_command.trim().is_empty()
            && !self.blue_light_state_command.trim().is_empty()
    }

    pub(super) fn blue_light_command_for_state(&self, enabled: bool) -> Option<&str> {
        if !self.blue_light_toggle_configured() {
            return None;
        }

        if enabled {
            Some(self.blue_light_enable_command.as_str())
        } else {
            Some(self.blue_light_disable_command.as_str())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blue_light_toggle_requires_enable_disable_and_state_commands() {
        let config = BrightnessConfig {
            blue_light_enable_command: "enable-night-mode".to_string(),
            blue_light_disable_command: "disable-night-mode".to_string(),
            blue_light_state_command: "check-night-mode".to_string(),
        };

        assert!(config.blue_light_toggle_configured());
    }

    #[test]
    fn blue_light_toggle_is_unconfigured_when_any_command_is_missing() {
        let config = BrightnessConfig {
            blue_light_enable_command: "enable-night-mode".to_string(),
            blue_light_disable_command: String::new(),
            blue_light_state_command: "check-night-mode".to_string(),
        };

        assert!(!config.blue_light_toggle_configured());
    }

    #[test]
    fn blue_light_command_for_state_picks_enable_or_disable_command() {
        let config = BrightnessConfig {
            blue_light_enable_command: "enable-night-mode".to_string(),
            blue_light_disable_command: "disable-night-mode".to_string(),
            blue_light_state_command: "check-night-mode".to_string(),
        };

        assert_eq!(
            config.blue_light_command_for_state(true),
            Some("enable-night-mode")
        );
        assert_eq!(
            config.blue_light_command_for_state(false),
            Some("disable-night-mode")
        );
    }
}
