use relm4::{ComponentSender, gtk};

use super::{Bar, battery, clock, layout::BarItem};

pub(super) fn render_item(bar: &Bar, item: BarItem, container: &gtk::Box) {
    match item {
        BarItem::Workspaces => bar.render_workspace_row(container),
        BarItem::Clock => clock::render(container, &bar.clock_text),
        BarItem::Battery => battery::render(container, &bar.battery_text),
    }
}

pub(super) fn start_item(
    item: BarItem,
    sender: &ComponentSender<Bar>
) -> relm4::JoinHandle<()> {
    match item {
        BarItem::Workspaces => {
            crate::niri::start_workspace_watcher(sender.input_sender().clone())
        }
        BarItem::Clock => {
            clock::start(sender.input_sender().clone())
        }
        BarItem::Battery => {
            battery::start(sender.input_sender().clone())
        }
    }
}

pub(super) fn initial_clock_text() -> String {
    clock::initial_text()
}

pub(super) fn initial_battery_text() -> String {
    battery::initial_text()
}
