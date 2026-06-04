use gtk::gdk;
use gtk::prelude::*;
use relm4::prelude::*;

use super::{Shell, monitors};
use crate::{bar, config::AppConfig};

pub(super) struct RunningBar {
    pub(super) key: String,
    pub(super) controller: Controller<bar::Bar>,
}

struct DesiredBar {
    key: String,
    config: crate::config::BarConfig,
    monitor: gdk::Monitor,
}

impl Shell {
    pub(super) fn update_focused_monitor(&mut self, state: &bar::state::BarItemState) {
        let bar::state::BarItemState::Workspaces(bar::state::WorkspaceState::Ready(workspaces)) =
            state
        else {
            return;
        };

        self.focused_monitor_connector = workspaces
            .iter()
            .find(|workspace| workspace.is_focused)
            .and_then(|workspace| workspace.output.clone());
    }

    fn desired_bars(&self) -> Vec<DesiredBar> {
        let mut desired = Vec::new();

        for bar_config in &self.config.bars {
            let Some(name) = bar_name(bar_config) else {
                tracing::error!("Skipping bar wthout a name");
                continue;
            };

            for monitor in monitors::target(bar_config) {
                let Some(connector) = monitors::connector(&monitor) else {
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

    pub(super) fn reconcile_bars(&mut self) {
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
                if let Some(running_bar) = Self::launch_bar(
                    &self.config,
                    &desired_bar.config,
                    desired_bar.monitor,
                    self.services.clone(),
                ) {
                    self.send_item_states_to_bar(&running_bar);
                    self.bars.push(running_bar);
                }

                continue;
            };

            let init = bar::BarInit::from_config(
                &self.config,
                Some(&desired_bar.config),
                None,
                self.services.clone(),
            );

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

    fn launch_bar(
        app_config: &AppConfig,
        bar_config: &crate::config::BarConfig,
        monitor: gdk::Monitor,
        services: crate::services::ShellServices,
    ) -> Option<RunningBar> {
        let Some(name) = bar_name(bar_config) else {
            tracing::error!("Skipping bar without a name");
            return None;
        };

        let Some(connector) = monitors::connector(&monitor) else {
            tracing::error!("Skipping bar {name} on monitor without connector");
            return None;
        };

        let key = running_bar_key(&name, &connector);

        tracing::info!("Launching bar {key}");

        let init = bar::BarInit::from_config(app_config, Some(bar_config), Some(monitor), services);
        let controller = bar::Bar::builder().launch(init).detach();

        Some(RunningBar { key, controller })
    }

    pub(super) fn send_item_states_to_bar(&self, running_bar: &RunningBar) {
        for state in &self.item_states {
            let _ = running_bar
                .controller
                .sender()
                .send(bar::BarMsg::ItemStateChanged(state.clone()));
        }
    }
}

fn bar_name(config: &crate::config::BarConfig) -> Option<String> {
    config.name.clone()
}

fn running_bar_key(bar_name: &str, monitor_connector: &str) -> String {
    format!("{bar_name}@{monitor_connector}")
}
