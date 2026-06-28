use crate::bar::widgets::systray::view_model::SystrayItemSummary;
use crate::bar::widgets::updates::model::UpdatesSnapshot;
use crate::bar::widgets::volume::model::VolumeSnapshot;
use crate::bar::widgets::workspaces::model::WorkspaceSummary;
use crate::notifications::model::NotificationToast;
use wayle_battery::types::DeviceState;
use wayle_power_profiles::types::profile::PowerProfile;

#[derive(Clone, Debug)]
pub(crate) enum BarItemState {
    Workspaces(WorkspaceState),
    Battery(BatteryState),
    Clock(ClockState),
    Systray(SystrayState),
    Notifications(NotificationState),
    Updates(UpdatesState),
    Volume(VolumeState),
    Brightness(BrightnessState),
}

#[derive(Clone, Debug)]
pub(crate) enum WorkspaceState {
    Connecting,
    Ready(Vec<WorkspaceSummary>),
    Unavailable(String),
    UpdatesStopped,
}

#[derive(Clone, Debug)]
pub(crate) enum BatteryState {
    Ready(BatterySnapshot),
    Unavailable,
}

#[derive(Clone, Debug)]
pub(crate) struct BatterySnapshot {
    pub(crate) percentage: f64,
    pub(crate) state: DeviceState,
    pub(crate) energy_rate: f64,
    pub(crate) time_to_empty: i64,
    pub(crate) time_to_full: i64,
    pub(crate) capacity: f64,
    pub(crate) active_profile: Option<PowerProfile>,
    pub(crate) available_profiles: Vec<PowerProfile>,
}

#[derive(Clone, Debug)]
pub(crate) enum ClockState {
    Ready,
}

#[derive(Clone)]
pub(crate) enum SystrayState {
    Ready(Vec<SystrayItemSummary>),
    Unavailable,
}

impl std::fmt::Debug for SystrayState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready(items) => write!(f, "Ready({} items(s))", items.len()),
            Self::Unavailable => f.write_str("Unavailable"),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum NotificationState {
    Ready(Vec<NotificationToast>),
    Unavailable,
}

impl BarItemState {
    pub(crate) fn widget_id(&self) -> &'static str {
        match self {
            Self::Workspaces(_) => "workspaces",
            Self::Battery(_) => "battery",
            Self::Clock(_) => "clock",
            Self::Systray(_) => "systray",
            Self::Notifications(_) => "notifications",
            Self::Updates(_) => "updates",
            Self::Volume(_) => "volume",
            Self::Brightness(_) => "brightness",
        }
    }

    pub(crate) fn same_widget_as(&self, other: &Self) -> bool {
        self.widget_id() == other.widget_id()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum UpdatesState {
    Ready(UpdatesSnapshot),
    Unavailable(String),
}

#[derive(Clone, Debug)]
pub(crate) enum VolumeState {
    Ready(VolumeSnapshot),
    Unavailable(String),
}

#[derive(Clone, Debug)]
pub(crate) enum BrightnessState {
    Ready(BrightnessSnapshot),
    Unavailable(String),
}

#[derive(Clone, Debug)]
pub(crate) struct BrightnessSnapshot {
    pub(crate) percent: f64,
}
