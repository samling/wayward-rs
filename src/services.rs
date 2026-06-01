use relm4::ComponentSender;

use crate::shell::Shell;

pub(crate) fn initial_item_states() -> Vec<crate::bar::state::BarItemState> {
    crate::bar::registry::WIDGETS
        .iter()
        .filter_map(|widget| widget.initial_state())
        .collect()
}

pub(crate) fn start_all(sender: &ComponentSender<Shell>) {
    let input_sender = sender.input_sender().clone();

    for widget in crate::bar::registry::WIDGETS {
        widget.start(input_sender.clone());
    }
}
