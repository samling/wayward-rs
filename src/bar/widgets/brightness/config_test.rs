use super::*;

#[test]
fn sunsetr_config_defaults_to_default_and_day_presets() {
    let config = BrightnessConfig::default();

    assert_eq!(config.sunsetr.automatic_preset, "default");
    assert_eq!(config.sunsetr.paused_preset, "day");
}
