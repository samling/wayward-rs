use crate::bar::state::BatterySnapshot;

use super::format::{
    battery_energy_rate_text, battery_health_text, battery_icon_name, battery_percentage_text, initial_text,
};

#[derive(Clone, Debug)]
pub(super) struct BatteryViewModel {
    pub(super) icon_name: &'static str,
    pub(super) percentage_text: String,
    pub(super) energy_rate_text: String,
    pub(super) health_text: String,
    pub(super) meter_value: f64,
}

impl BatteryViewModel {
    pub(super) fn unavailable() -> Self {
        Self {
            icon_name: "battery-missing-symbolic",
            percentage_text: initial_text(),
            energy_rate_text: String::new(),
            health_text: String::new(),
            meter_value: 0.0,
        }
    }

    pub(super) fn from_snapshot(snapshot: &BatterySnapshot) -> Self {
        Self {
            icon_name: battery_icon_name(snapshot.percentage, snapshot.state),
            percentage_text: battery_percentage_text(snapshot.percentage),
            energy_rate_text: battery_energy_rate_text(snapshot.energy_rate),
            health_text: battery_health_text(snapshot.capacity),
            meter_value: snapshot.percentage.clamp(0.0, 100.0),
        }
    }
}