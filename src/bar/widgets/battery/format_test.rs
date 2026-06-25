use super::*;

#[test]
fn battery_percentage_text_rounds_to_whole_percent() {
    assert_eq!(battery_percentage_text(87.4), "87%");
    assert_eq!(battery_percentage_text(87.5), "88%");
}

#[test]
fn battery_energy_rate_text_formats_watts_directly() {
    assert_eq!(battery_energy_rate_text(6.24), "6.2W");
    assert_eq!(battery_energy_rate_text(0.04), "0.0W");
}

#[test]
fn battery_icon_name_uses_charged_icon_for_fully_charged() {
    assert_eq!(
        battery_icon_name(100.0, DeviceState::FullyCharged),
        "battery-level-100-charged-symbolic"
    );
}

#[test]
fn battery_icon_name_uses_charging_icons_for_charging_state() {
    assert_eq!(
        battery_icon_name(84.0, DeviceState::Charging),
        "battery-level-80-charging-symbolic"
    );
}

#[test]
fn battery_icon_name_uses_level_icons_for_other_states() {
    assert_eq!(
        battery_icon_name(26.0, DeviceState::Discharging),
        "battery-level-30-symbolic"
    );
}
