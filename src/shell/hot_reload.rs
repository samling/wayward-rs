use gtk::prelude::*;
use relm4::gtk;
use relm4::prelude::ComponentSender;

use super::{Shell, ShellMsg};
use crate::config::AppConfig;

impl Shell {
    pub(super) fn start_config_hot_reload(sender: &ComponentSender<Self>) {
        let Some(dir) = crate::config::config_dir() else {
            tracing::info!("Could not determine config directory, config hot reload disabled");
            return;
        };
        let Some(path) = crate::config::config_path() else {
            tracing::info!("Could not determine config path, config hot reload disabled");
            return;
        };

        let input_sender = sender.input_sender().clone();

        crate::file_watch::start_debounced_file_watch("config", dir, path, move || {
            if input_sender
                .send(ShellMsg::ConfigChanged(AppConfig::load()))
                .is_err()
            {
                return;
            }

            tracing::info!("Reloaded config");
        });
    }

    pub(super) fn start_monitor_watch(sender: &ComponentSender<Self>) {
        let Some(display) = gtk::gdk::Display::default() else {
            tracing::error!("Could not determine default display, monitor hot reload disabled");
            return;
        };

        let monitors = display.monitors();
        let input_sender = sender.input_sender().clone();

        monitors.connect_items_changed(move |_, _position, _removed, _added| {
            if input_sender.send(ShellMsg::MonitorsChanged).is_err() {
                tracing::error!("Failed to send monitor change message");
            }
        });
    }
}
