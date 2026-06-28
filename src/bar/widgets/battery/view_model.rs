use crate::bar::state::BatterySnapshot;

use super::format::{
    battery_energy_rate_text, battery_health_text, battery_icon_name, battery_percentage_text,
    battery_state_text, battery_time_remaining_duration_text, battery_time_remaining_label,
    battery_time_remaining_text, initial_text,
};

#[derive(Clone, Debug)]
pub(super) struct BatteryViewModel {
    pub(super) icon_name: &'static str,
    pub(super) percentage_text: String,
    pub(super) energy_rate_text: String,
    pub(super) health_text: String,
    pub(super) state_text: String,
    pub(super) time_remaining_label: &'static str,
    pub(super) time_remaining_text: String,
    pub(super) tooltip_text: Option<String>,
    pub(super) meter_value: f64,
}

impl BatteryViewModel {
    pub(super) fn unavailable() -> Self {
        Self {
            icon_name: "battery-missing-symbolic",
            percentage_text: initial_text(),
            energy_rate_text: String::new(),
            health_text: String::new(),
            state_text: "Unavailable".to_string(),
            time_remaining_label: "Time remaining",
            time_remaining_text: "Unavailable".to_string(),
            tooltip_text: None,
            meter_value: 0.0,
        }
    }

    pub(super) fn from_snapshot(snapshot: &BatterySnapshot) -> Self {
        let time_remaining_text = battery_time_remaining_text(
            snapshot.state,
            snapshot.time_to_empty,
            snapshot.time_to_full,
        );
        let time_remaining_duration_text = battery_time_remaining_duration_text(
            snapshot.state,
            snapshot.time_to_empty,
            snapshot.time_to_full,
        );

        Self {
            icon_name: battery_icon_name(snapshot.percentage, snapshot.state),
            percentage_text: battery_percentage_text(snapshot.percentage),
            energy_rate_text: battery_energy_rate_text(snapshot.energy_rate),
            health_text: battery_health_text(snapshot.capacity),
            state_text: battery_state_text(snapshot.state),
            time_remaining_label: battery_time_remaining_label(snapshot.state),
            time_remaining_text: time_remaining_duration_text
                .unwrap_or_else(|| "Unavailable".to_string()),
            tooltip_text: time_remaining_text,
            meter_value: snapshot.percentage.clamp(0.0, 100.0),
        }
    }
}
