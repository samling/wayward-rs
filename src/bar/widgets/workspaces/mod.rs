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

impl BarWidget for WorkspacesWidget {
    fn id(&self) -> &'static str {
        ID
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
