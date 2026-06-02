pub(crate) mod model;
mod render;
pub(crate) mod service;

use relm4::gtk;
use relm4::gtk::glib::object::Cast;

use crate::bar::BarMsg;
use crate::bar::state::{BarItemState, WorkspaceState};
use crate::bar::widget::{BarContext, BarWidget, BarWidgetRuntime, WidgetInstance};
use crate::services::ShellServices;
use crate::shell::ShellMsg;

use self::render::{render_status, render_workspace_state};

struct WorkspacesRuntime {
    root: gtk::Box,
}

impl BarWidgetRuntime for WorkspacesRuntime {
    fn root(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        let BarItemState::Workspaces(state) = state else {
            return;
        };

        render_workspace_state(&self.root, state, context.monitor_connector.as_deref());
    }
}

pub(crate) struct WorkspacesWidget;

impl BarWidget for WorkspacesWidget {
    fn id(&self) -> &'static str {
        "workspaces"
    }

    fn build(
        &self,
        _instance: &WidgetInstance,
        _sender: &relm4::Sender<BarMsg>,
        _services: &crate::services::ShellServices,
    ) -> Box<dyn BarWidgetRuntime> {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        render_status(&row, "Connecting to Niri");

        Box::new(WorkspacesRuntime { root: row })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Workspaces(WorkspaceState::Connecting))
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
