use super::widget::BarWidget;
use super::widgets::battery::BatteryWidget;
use super::widgets::clock::ClockWidget;
use super::widgets::workspaces::WorkspacesWidget;

static BATTERY: BatteryWidget = BatteryWidget;
static CLOCK: ClockWidget = ClockWidget;
static WORKSPACES: WorkspacesWidget = WorkspacesWidget;

pub(crate) static WIDGETS: &[&dyn BarWidget] = &[&WORKSPACES, &CLOCK, &BATTERY];

pub(crate) fn widget_by_id(id: &str) -> Option<&'static dyn BarWidget> {
    WIDGETS.iter().copied().find(|widget| widget.id() == id)
}
