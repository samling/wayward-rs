use crate::workspace::WorkspaceSummary;

#[derive(Clone, Debug)]
pub(crate) enum BarItemState {
    Workspaces(WorkspaceState),
    Battery(BatteryState),
    Clock(ClockState),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BarItemStateKey {
    Workspaces,
    Battery,
    Clock,
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
    pub(crate) fn key(&self) -> BarItemStateKey {
        match self {
            Self::Workspaces(_) => BarItemStateKey::Workspaces,
            Self::Battery(_) => BarItemStateKey::Battery,
            Self::Clock(_) => BarItemStateKey::Clock,
        }
    }

    pub(crate) fn same_item_as(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}
