use super::widget::{BarWidget, WidgetEvent};
use super::widgets::battery::BatteryWidget;
use super::widgets::clock::ClockWidget;
use super::widgets::systray::{self, SystrayWidget};
use super::widgets::workspaces::{self, WorkspacesWidget};

static BATTERY: BatteryWidget = BatteryWidget;
static CLOCK: ClockWidget = ClockWidget;
static SYSTRAY: SystrayWidget = SystrayWidget;
static WORKSPACES: WorkspacesWidget = WorkspacesWidget;

pub(crate) static WIDGETS: &[&dyn BarWidget] = &[&WORKSPACES, &CLOCK, &BATTERY, &SYSTRAY];

pub(crate) fn widget_by_id(id: &str) -> Option<&'static dyn BarWidget> {
    WIDGETS.iter().copied().find(|widget| widget.id() == id)
}

pub(crate) fn handle_widget_event(event: WidgetEvent) {
    match event.widget_id {
        systray::ID => {
            systray::service::handle_event(event);
        }
        workspaces::ID => {
            workspaces::service::handle_event(event);
        }
        unknown => {
            tracing::warn!("No widget event handler registered for {unknown}")
        }
    }
}
