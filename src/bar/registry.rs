use crate::services::ShellServices;

use super::widget::{BarWidget, WidgetEvent};
use super::widgets;

pub(crate) static WIDGETS: &[&dyn BarWidget] = &[
    &widgets::action_menu::WIDGET,
    &widgets::battery::WIDGET,
    &widgets::brightness::WIDGET,
    &widgets::clock::WIDGET,
    &widgets::notifications::WIDGET,
    &widgets::systray::WIDGET,
    &widgets::updates::WIDGET,
    &widgets::volume::WIDGET,
    &widgets::workspaces::WIDGET,
];

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
