use gtk::prelude::*;
use relm4::gtk;

use gtk::gdk;

pub(super) fn available() -> Vec<gdk::Monitor> {
    let Some(display) = gdk::Display::default() else {
        tracing::error!("Could not determine default display");
        return Vec::new();
    };

    let monitors = display.monitors();
    let mut available = Vec::new();

    for index in 0..monitors.n_items() {
        let Some(item) = monitors.item(index) else {
            continue;
        };

        let Ok(monitor) = item.downcast::<gdk::Monitor>() else {
            continue;
        };

        available.push(monitor);
    }

    available
}

pub(super) fn available_connectors() -> Vec<String> {
    available().iter().filter_map(connector).collect()
}

pub(super) fn target(bar_config: &crate::config::BarConfig) -> Vec<gdk::Monitor> {
    let available_monitors = available();

    let Some(configured_monitors) = &bar_config.monitors else {
        return available_monitors;
    };

    let mut targets = Vec::new();

    for configured_monitor in configured_monitors {
        let Some(monitor) = available_monitors
            .iter()
            .find(|monitor| connector(monitor).as_deref() == Some(configured_monitor))
        else {
            tracing::error!("Configured monitor not found: {configured_monitor}");
            continue;
        };

        targets.push(monitor.clone());
    }

    targets
}

pub(super) fn has_without_connector() -> bool {
    available()
        .iter()
        .any(|monitor| connector(monitor).is_none())
}

pub(super) fn connector(monitor: &gdk::Monitor) -> Option<String> {
    monitor.connector().map(|connector| connector.to_string())
}
