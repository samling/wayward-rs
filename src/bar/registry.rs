use crate::services::ShellServices;

use super::widget::{BarWidget, WidgetEvent};
use super::widgets::battery::BatteryWidget;
use super::widgets::clock::ClockWidget;
use super::widgets::systray::SystrayWidget;
use super::widgets::workspaces::WorkspacesWidget;

static BATTERY: BatteryWidget = BatteryWidget;
static CLOCK: ClockWidget = ClockWidget;
static SYSTRAY: SystrayWidget = SystrayWidget;
static WORKSPACES: WorkspacesWidget = WorkspacesWidget;

pub(crate) static WIDGETS: &[&dyn BarWidget] = &[&WORKSPACES, &CLOCK, &BATTERY, &SYSTRAY];

pub(crate) fn widget_by_id(id: &str) -> Option<&'static dyn BarWidget> {
    WIDGETS.iter().copied().find(|widget| widget.id() == id)
}

pub(crate) fn handle_widget_event(event: WidgetEvent, services: &ShellServices) {
    let Some(widget) = widget_by_id(event.widget_id) else {
        tracing::warn!("No widget registered for event {}", event.widget_id);
        return;
    };

    widget.handle_event(event, services);
}
