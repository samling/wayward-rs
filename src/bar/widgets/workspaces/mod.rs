mod component;
mod config;
mod indicator;
pub(crate) mod model;
mod render;
pub(crate) mod service;

use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::{Component, ComponentController, Controller};

use crate::bar::state::{BarItemState, WorkspaceState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetBuildContext, WidgetEvent, WidgetInstance,
};
use crate::services::ShellServices;
use crate::settings_spec::{
    ChoiceOption, ChoiceSpec, NumberSpec, SettingSpec, SettingsSectionSpec, StringSpec,
    table_string, table_u16,
};
use crate::shell::ShellMsg;

use self::component::{WorkspacesComponent, WorkspacesInit, WorkspacesInput};

pub(crate) const ID: &str = "workspaces";

struct WorkspacesRuntime {
    controller: Controller<WorkspacesComponent>,
}

impl BarWidgetRuntime for WorkspacesRuntime {
    fn root(&self) -> gtk::Widget {
        self.controller.widget().clone().upcast()
    }

    fn set_context(&mut self, context: &BarContext) {
        self.controller.emit(WorkspacesInput::SetPlacement {
            edge: context.edge,
            monitor_connector: context.monitor_connector.clone(),
        });
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        let BarItemState::Workspaces(state) = state else {
            return;
        };

        self.controller.emit(WorkspacesInput::SetState {
            state: state.clone(),
            monitor_connector: context.monitor_connector.clone(),
        });
    }
}

pub(crate) struct WorkspacesWidget;

pub(crate) static WIDGET: WorkspacesWidget = WorkspacesWidget;

impl BarWidget for WorkspacesWidget {
    fn id(&self) -> &'static str {
        ID
    }

    fn settings_sections(
        &self,
        config: &toml::value::Table,
    ) -> Vec<crate::settings_spec::SettingsSectionSpec> {
        vec![SettingsSectionSpec {
            title: "Config".to_string(),
            settings: vec![
                SettingSpec::String(StringSpec {
                    label: "Label format",
                    description: Some("%I index · %T title · %L title or index · %% literal %"),
                    path: &["widgets", "workspaces", "label_format"],
                    value: table_string(config, &["label_format"]),
                    default: "%L",
                }),
                SettingSpec::Choice(ChoiceSpec {
                    label: "Indicator effect",
                    description: None,
                    path: &["widgets", "workspaces", "indicator_effect"],
                    value: table_string(config, &["indicator_effect"]),
                    default: "ease",
                    options: &[
                        ChoiceOption {
                            value: "none",
                            label: "None",
                        },
                        ChoiceOption {
                            value: "slide",
                            label: "Slide",
                        },
                        ChoiceOption {
                            value: "ease",
                            label: "Ease",
                        },
                    ],
                }),
                SettingSpec::Number(NumberSpec {
                    label: "Indicator duration (ms)",
                    description: None,
                    path: &["widgets", "workspaces", "indicator_duration_ms"],
                    value: table_u16(config, "indicator_duration_ms"),
                    default: 160,
                    min: 0.0,
                    max: 1000.0,
                    step: 10.0,
                }),
            ],
        }]
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime> {
        let config = instance.config_as::<config::WorkspacesConfig>();
        let controller = WorkspacesComponent::builder()
            .launch(WorkspacesInit {
                edge: context.bar.edge,
                monitor_connector: context.bar.monitor_connector.clone(),
                config,
                instance_class: instance.instance_css_class(),
                sender: context.sender.clone(),
            })
            .detach();
        Box::new(WorkspacesRuntime { controller })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Workspaces(WorkspaceState::Connecting))
    }

    fn handle_event(&self, event: WidgetEvent, services: &ShellServices) {
        service::handle_event(event, services.niri.clone());
    }

    fn start(
        &self,
        sender: relm4::Sender<ShellMsg>,
        services: &ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        Some(service::start_workspace_watcher(
            sender,
            services.niri.clone(),
        ))
    }
}
