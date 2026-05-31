use crate::workspace::WorkspaceSummary;

#[derive(Clone, Debug)]
pub(crate) enum BarItemState {
    Workspaces(WorkspaceState),
    Battery(BatteryState),
    Clock(ClockState),
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
    Ready(String),
    Unavailable,
}

#[derive(Clone, Debug)]
pub(crate) enum ClockState {
    Ready(String),
}

impl BarItemState {
    pub(crate) fn same_item_as(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Workspaces(_), Self::Workspaces(_))
                | (Self::Battery(_), Self::Battery(_))
                | (Self::Clock(_), Self::Clock(_))
        )
    }
}
