use relm4::gtk;

use super::{Bar, battery, clock, layout::BarItem};

pub(super) fn render_item(bar: &Bar, item: BarItem, container: &gtk::Box) {
    match item {
        BarItem::Workspaces => bar.render_workspace_row(container),
        BarItem::Clock => clock::render(container, &bar.clock_text),
        BarItem::Battery => battery::render(container, &bar.battery_text),
    }
}
pub(super) fn initial_clock_text() -> String {
    clock::initial_text()
}

pub(super) fn initial_battery_text() -> String {
    battery::initial_text()
}
