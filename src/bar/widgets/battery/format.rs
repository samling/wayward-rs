use wayle_battery::types::DeviceState;

pub(super) fn initial_text() -> String {
    "NaN".to_string()
}

pub(super) fn battery_percentage_text(percentage: f64) -> String {
    format!("{percentage:.0}%")
}

pub(super) fn battery_energy_rate_text(energy_rate: f64) -> String {
    format!("{energy_rate:.1}W")
}

pub(super) fn battery_icon_name(percentage: f64, state: DeviceState) -> &'static str {
    let level = ((percentage / 10.0).round() as i32 * 10).clamp(0, 100);

    match state {
        DeviceState::FullyCharged => "battery-level-100-charged-symbolic",
        DeviceState::Charging => charging_battery_icon_name(level),
        _ => discharging_battery_icon_name(level),
    }
}

fn charging_battery_icon_name(level: i32) -> &'static str {
    match level {
        100 => "battery-level-100-charging-symbolic",
        90 => "battery-level-90-charging-symbolic",
        80 => "battery-level-80-charging-symbolic",
        70 => "battery-level-70-charging-symbolic",
        60 => "battery-level-60-charging-symbolic",
        50 => "battery-level-50-charging-symbolic",
        40 => "battery-level-40-charging-symbolic",
        30 => "battery-level-30-charging-symbolic",
        20 => "battery-level-20-charging-symbolic",
        10 => "battery-level-10-charging-symbolic",
        _ => "battery-level-0-charging-symbolic",
    }
}

fn discharging_battery_icon_name(level: i32) -> &'static str {
    match level {
        100 => "battery-level-100-symbolic",
        90 => "battery-level-90-symbolic",
        80 => "battery-level-80-symbolic",
        70 => "battery-level-70-symbolic",
        60 => "battery-level-60-symbolic",
        50 => "battery-level-50-symbolic",
        40 => "battery-level-40-symbolic",
        30 => "battery-level-30-symbolic",
        20 => "battery-level-20-symbolic",
        10 => "battery-level-10-symbolic",
        _ => "battery-level-0-symbolic",
    }
}

#[cfg(test)]
mod tests {
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
}
