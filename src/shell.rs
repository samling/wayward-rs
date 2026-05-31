use gtk::prelude::*;
use relm4::gtk;
use relm4::prelude::*;

use crate::{bar, config::AppConfig};

pub struct Shell {
    bars: Vec<RunningBar>,
}

#[derive(Debug)]
pub enum ShellMsg {
    ConfigChanged(AppConfig),
}

struct RunningBar {
    name: String,
    controller: Controller<bar::Bar>,
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

    fn launch_bar(bar_config: &crate::config::BarConfig) -> Option<RunningBar> {
        let Some(name) = bar_name(bar_config) else {
            tracing::error!("Skipping bar without a name");
            return None;
        };

        tracing::info!("Launching bar {name}");

        let init = bar::BarInit::from_config(Some(bar_config));
        let controller = bar::Bar::builder().launch(init).detach();

        Some(RunningBar { name, controller })
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

        let bars = config.bars.iter().filter_map(Self::launch_bar).collect();

        Self::start_config_hot_reload(&sender);

        let model = Shell { bars };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ShellMsg::ConfigChanged(config) => {
                self.bars.retain(|running_bar| {
                    let still_configured = config
                        .bars
                        .iter()
                        .filter_map(bar_name)
                        .any(|name| name == running_bar.name);

                    if !still_configured {
                        tracing::info!("Removing bar {}", running_bar.name);
                        running_bar.controller.widget().close();
                    }

                    still_configured
                });
                for bar_config in &config.bars {
                    let Some(name) = bar_name(bar_config) else {
                        tracing::error!("Skipping bar without a name");
                        continue;
                    };

                    let Some(running_bar) = self.bars.iter().find(|bar| bar.name == name) else {
                        if let Some(running_bar) = Self::launch_bar(bar_config) {
                            self.bars.push(running_bar);
                        }
                        continue;
                    };

                    let init = bar::BarInit::from_config(Some(bar_config));

                    if running_bar
                        .controller
                        .sender()
                        .send(bar::BarMsg::LayoutChanged { layout: init.layout, edge: init.edge })
                        .is_err()
                    {
                        tracing::error!("Failed to send layout update to bar {name}")
                    }
                }
                tracing::info!("Config changed: {} bar(s)", config.bars.len());
            }
        }
    }
}

fn bar_name(config: &crate::config::BarConfig) -> Option<String> {
    config.name.clone()
}