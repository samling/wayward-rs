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

pub(super) fn battery_health_text(capacity: f64) -> String {
    format!("{capacity:.0}%")
}

pub(super) fn battery_time_remaining_text(
    state: DeviceState,
    time_to_empty: i64,
    time_to_full: i64,
) -> Option<String> {
    battery_time_remaining_duration_text(state, time_to_empty, time_to_full)
        .map(|duration| format!("{}: {duration}", battery_time_remaining_label(state)))
}

pub(super) fn battery_time_remaining_label(state: DeviceState) -> &'static str {
    match state {
        DeviceState::Discharging => "Time to empty",
        DeviceState::Charging => "Time to full",
        _ => "Time remaining",
    }
}

pub(super) fn battery_time_remaining_duration_text(
    state: DeviceState,
    time_to_empty: i64,
    time_to_full: i64,
) -> Option<String> {
    match state {
        DeviceState::Discharging => format_duration(time_to_empty),
        DeviceState::Charging => format_duration(time_to_full),
        _ => None,
    }
}

fn format_duration(seconds: i64) -> Option<String> {
    if seconds <= 0 {
        return None;
    }

    let total_minutes = ((seconds + 30) / 60).max(1);
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    match (hours, minutes) {
        (0, minutes) => Some(format!("{minutes}m")),
        (hours, 0) => Some(format!("{hours}h")),
        (hours, minutes) => Some(format!("{hours}h {minutes}m")),
    }
}

pub(super) fn battery_state_text(state: DeviceState) -> String {
    state.to_string()
}

pub(super) fn battery_icon_names(percentage: f64, state: DeviceState) -> &'static [&'static str] {
    let level = ((percentage / 10.0).round() as i32 * 10).clamp(0, 100);

    match state {
        DeviceState::FullyCharged => &["battery-100-charged", "battery-level-100-charged-symbolic"],
        DeviceState::Charging => charging_battery_icon_names(level),
        _ => discharging_battery_icon_names(level),
    }
}

fn charging_battery_icon_names(level: i32) -> &'static [&'static str] {
    match level {
        100 => &[
            "battery-100-charging",
            "battery-level-100-charging-symbolic",
        ],
        90 => &["battery-090-charging", "battery-level-90-charging-symbolic"],
        80 => &["battery-080-charging", "battery-level-80-charging-symbolic"],
        70 => &["battery-070-charging", "battery-level-70-charging-symbolic"],
        60 => &["battery-060-charging", "battery-level-60-charging-symbolic"],
        50 => &["battery-050-charging", "battery-level-50-charging-symbolic"],
        40 => &["battery-040-charging", "battery-level-40-charging-symbolic"],
        30 => &["battery-030-charging", "battery-level-30-charging-symbolic"],
        20 => &["battery-020-charging", "battery-level-20-charging-symbolic"],
        10 => &["battery-010-charging", "battery-level-10-charging-symbolic"],
        _ => &["battery-000-charging", "battery-level-0-charging-symbolic"],
    }
}

fn discharging_battery_icon_names(level: i32) -> &'static [&'static str] {
    match level {
        100 => &["battery-100", "battery-level-100-symbolic"],
        90 => &["battery-090", "battery-level-90-symbolic"],
        80 => &["battery-080", "battery-level-80-symbolic"],
        70 => &["battery-070", "battery-level-70-symbolic"],
        60 => &["battery-060", "battery-level-60-symbolic"],
        50 => &["battery-050", "battery-level-50-symbolic"],
        40 => &["battery-040", "battery-level-40-symbolic"],
        30 => &["battery-030", "battery-level-30-symbolic"],
        20 => &["battery-020", "battery-level-20-symbolic"],
        10 => &["battery-010", "battery-level-10-symbolic"],
        _ => &["battery-000", "battery-level-0-symbolic"],
    }
}

#[cfg(test)]
#[path = "format_test.rs"]
mod tests;
