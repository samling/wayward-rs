use relm4::ComponentSender;

use crate::shell::Shell;

pub(crate) fn initial_item_states() -> Vec<crate::bar::state::BarItemState> {
    use crate::bar::state::{BarItemState, BatteryState, ClockState, WorkspaceState};

    vec![
        BarItemState::Workspaces(WorkspaceState::Connecting),
        BarItemState::Battery(BatteryState::Unavailable),
        BarItemState::Clock(ClockState::Ready(crate::bar::clock::initial_text())),
    ]
}

pub(crate) fn start_all(sender: &ComponentSender<Shell>) {
    crate::niri::start_workspace_watcher(sender.input_sender().clone());
    crate::bar::battery::start(sender.input_sender().clone());
    crate::bar::clock::start(sender.input_sender().clone());
}