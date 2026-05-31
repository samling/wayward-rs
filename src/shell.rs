use gtk::gdk;
use gtk::prelude::*;
use relm4::gtk;
use relm4::prelude::*;

use crate::{bar, config::AppConfig};

pub struct Shell {
    bars: Vec<RunningBar>,
    config: AppConfig,
    item_states: Vec<bar::state::BarItemState>,
}

#[derive(Debug)]
pub enum ShellMsg {
    ConfigChanged(AppConfig),
    MonitorsChanged,
    ReconcileMonitors,
    ItemStateChanged(bar::state::BarItemState),
}

struct RunningBar {
    key: String,
    controller: Controller<bar::Bar>,
}

struct DesiredBar {
    key: String,
    config: crate::config::BarConfig,
    monitor: gdk::Monitor,
}

impl Shell {
    fn start_config_hot_reload(sender: &ComponentSender<Self>) {
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

    fn start_monitor_watch(sender: &ComponentSender<Self>) {
        let Some(display) = gdk::Display::default() else {
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

    fn desired_bars(&self) -> Vec<DesiredBar> {
        let mut desired = Vec::new();

        for bar_config in &self.config.bars {
            let Some(name) = bar_name(bar_config) else {
                tracing::error!("Skipping bar wthout a name");
                continue;
            };

            for monitor in Self::target_monitors(bar_config) {
                let Some(connector) = monitor_connector(&monitor) else {
                    tracing::error!("Skipping bar {name} on monitor without connector");
                    continue;
                };

                let key = running_bar_key(&name, &connector);

                if desired.iter().any(|bar: &DesiredBar| bar.key == key) {
                    tracing::error!("Duplicate bar key {key}, skipping duplicate");
                    continue;
                }

                desired.push(DesiredBar {
                    key,
                    config: bar_config.clone(),
                    monitor,
                });
            }
        }

        desired
    }

    fn reconcile_bars(&mut self) {
        let desired_bars = self.desired_bars();

        self.bars.retain(|running_bar| {
            let still_configured = desired_bars
                .iter()
                .any(|desired_bar| desired_bar.key == running_bar.key);

            if !still_configured {
                tracing::info!("Removing bar{}", running_bar.key);
                running_bar.controller.widget().close();
            }

            still_configured
        });

        for desired_bar in desired_bars {
            let Some(running_bar) = self.bars.iter().find(|bar| bar.key == desired_bar.key) else {
                if let Some(running_bar) =
                    Self::launch_bar(&desired_bar.config, desired_bar.monitor)
                {
                    self.send_item_states_to_bar(&running_bar);
                    self.bars.push(running_bar);
                }

                continue;
            };

            let init = bar::BarInit::from_config(Some(&desired_bar.config), None);

            if running_bar
                .controller
                .sender()
                .send(bar::BarMsg::LayoutChanged {
                    layout: init.layout,
                    edge: init.edge,
                })
                .is_err()
            {
                tracing::error!("Failed to send layout update to bar {}", desired_bar.key);
            }
        }

        tracing::info!("Config changed: {} bar(s)", self.config.bars.len());
    }

    fn available_monitors() -> Vec<gdk::Monitor> {
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

    fn target_monitors(bar_config: &crate::config::BarConfig) -> Vec<gdk::Monitor> {
        let available_monitors = Self::available_monitors();

        let Some(configured_monitors) = &bar_config.monitors else {
            return available_monitors;
        };

        let mut targets = Vec::new();

        for configured_monitor in configured_monitors {
            let Some(monitor) = available_monitors
                .iter()
                .find(|monitor| monitor_connector(monitor).as_deref() == Some(configured_monitor))
            else {
                tracing::error!("Configured monitor not found: {configured_monitor}");
                continue;
            };

            targets.push(monitor.clone());
        }

        targets
    }

    fn has_monitor_without_connector() -> bool {
        Self::available_monitors()
            .iter()
            .any(|monitor| monitor_connector(monitor).is_none())
    }

    fn launch_bar(
        bar_config: &crate::config::BarConfig,
        monitor: gdk::Monitor,
    ) -> Option<RunningBar> {
        let Some(name) = bar_name(bar_config) else {
            tracing::error!("Skipping bar without a name");
            return None;
        };

        let Some(connector) = monitor_connector(&monitor) else {
            tracing::error!("Skipping bar {name} on monitor without connector");
            return None;
        };

        let key = running_bar_key(&name, &connector);

        tracing::info!("Launching bar {key}");

        let init = bar::BarInit::from_config(Some(bar_config), Some(monitor));
        let controller = bar::Bar::builder().launch(init).detach();

        Some(RunningBar { key, controller })
    }

    fn send_item_states_to_bar(&self, running_bar: &RunningBar) {
        for state in &self.item_states {
            let _ = running_bar
                .controller
                .sender()
                .send(bar::BarMsg::ItemStateChanged(state.clone()));
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for Shell {
    type Init = ();
    type Input = ShellMsg;
    type Output = ();

    view! {
        gtk::Window {
            set_visible: false,
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = AppConfig::load();

        let mut model = Shell {
            bars: Vec::new(),
            config,
            item_states: crate::services::initial_item_states(),
        };

        model.reconcile_bars();

        Self::start_config_hot_reload(&sender);
        Self::start_monitor_watch(&sender);
        crate::services::start_all(&sender);

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ShellMsg::ConfigChanged(config) => {
                self.config = config;
                self.reconcile_bars();
            }
            ShellMsg::MonitorsChanged => {
                tracing::info!("Monitors changed");

                let input_sender = _sender.input_sender().clone();

                gtk::glib::timeout_add_once(std::time::Duration::from_millis(500), move || {
                    let _ = input_sender.send(ShellMsg::ReconcileMonitors);
                });
            }
            ShellMsg::ReconcileMonitors => {
                self.reconcile_bars();

                if Self::has_monitor_without_connector() {
                    let input_sender = _sender.input_sender().clone();

                    gtk::glib::timeout_add_once(std::time::Duration::from_millis(500), move || {
                        let _ = input_sender.send(ShellMsg::ReconcileMonitors);
                    });
                }
            }
            ShellMsg::ItemStateChanged(state) => {
                self.item_states
                    .retain(|existing_state| !existing_state.same_item_as(&state));

                self.item_states.push(state.clone());

                for running_bar in &self.bars {
                    let _ = running_bar
                        .controller
                        .sender()
                        .send(bar::BarMsg::ItemStateChanged(state.clone()));
                }
            }
        }
    }
}

fn bar_name(config: &crate::config::BarConfig) -> Option<String> {
    config.name.clone()
}

fn monitor_connector(monitor: &gdk::Monitor) -> Option<String> {
    monitor.connector().map(|connector| connector.to_string())
}

fn running_bar_key(bar_name: &str, monitor_connector: &str) -> String {
    format!("{bar_name}@{monitor_connector}")
}
