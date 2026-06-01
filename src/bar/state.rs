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
    Ready,
}

impl BarItemState {
    pub(crate) fn widget_id(&self) -> &'static str {
        match self {
            Self::Workspaces(_) => "workspaces",
            Self::Battery(_) => "battery",
            Self::Clock(_) => "clock",
        }
    }

    pub(crate) fn same_widget_as(&self, other: &Self) -> bool {
        self.widget_id() == other.widget_id()
    }
}
