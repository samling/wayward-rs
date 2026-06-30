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
fn battery_time_remaining_text_formats_relevant_duration() {
    assert_eq!(
        battery_time_remaining_text(DeviceState::Discharging, 5400, 0),
        Some("Time to empty: 1h 30m".to_string())
    );
    assert_eq!(
        battery_time_remaining_label(DeviceState::Discharging),
        "Time to empty"
    );
    assert_eq!(
        battery_time_remaining_duration_text(DeviceState::Discharging, 5400, 0),
        Some("1h 30m".to_string())
    );

    assert_eq!(
        battery_time_remaining_text(DeviceState::Charging, 0, 2700),
        Some("Time to full: 45m".to_string())
    );
    assert_eq!(
        battery_time_remaining_label(DeviceState::Charging),
        "Time to full"
    );
    assert_eq!(
        battery_time_remaining_duration_text(DeviceState::Charging, 0, 2700),
        Some("45m".to_string())
    );
}

#[test]
fn battery_time_remaining_text_hides_missing_duration() {
    assert_eq!(
        battery_time_remaining_text(DeviceState::Discharging, 0, 0),
        None
    );
    assert_eq!(
        battery_time_remaining_text(DeviceState::FullyCharged, 0, 0),
        None
    );
}

#[test]
fn battery_icon_names_use_charged_icon_for_fully_charged() {
    assert_eq!(
        battery_icon_names(100.0, DeviceState::FullyCharged),
        &["battery-100-charged", "battery-level-100-charged-symbolic"]
    );
}

#[test]
fn battery_icon_names_use_charging_icons_for_charging_state() {
    assert_eq!(
        battery_icon_names(84.0, DeviceState::Charging),
        &["battery-080-charging", "battery-level-80-charging-symbolic"]
    );
}

#[test]
fn battery_icon_names_use_full_icon_for_high_charging_levels() {
    assert_eq!(
        battery_icon_names(99.0, DeviceState::Charging),
        &[
            "battery-100-charging",
            "battery-level-100-charging-symbolic"
        ]
    );
}

#[test]
fn battery_icon_names_use_mid_level_icon_when_discharging() {
    assert_eq!(
        battery_icon_names(68.0, DeviceState::Discharging),
        &["battery-070", "battery-level-70-symbolic"]
    );
}

#[test]
fn battery_icon_names_use_level_icons_for_other_states() {
    assert_eq!(
        battery_icon_names(26.0, DeviceState::Discharging),
        &["battery-030", "battery-level-30-symbolic"]
    );
}

#[test]
fn battery_icon_names_use_full_icon_for_high_discharging_levels() {
    assert_eq!(
        battery_icon_names(96.0, DeviceState::Discharging),
        &["battery-100", "battery-level-100-symbolic"]
    );
}
