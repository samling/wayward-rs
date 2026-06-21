use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub(super) struct BrightnessConfig {
    pub(super) sunsetr: SunsetrConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct SunsetrConfig {
    pub(super) automatic_preset: String,
    pub(super) paused_preset: String,
}

impl Default for SunsetrConfig {
    fn default() -> Self {
        Self {
            automatic_preset: "default".to_string(),
            paused_preset: "day".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sunsetr_config_defaults_to_default_and_day_presets() {
        let config = BrightnessConfig::default();

        assert_eq!(config.sunsetr.automatic_preset, "default");
        assert_eq!(config.sunsetr.paused_preset, "day");
    }
}
