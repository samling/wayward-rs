pub(crate) mod model;
pub(crate) mod service;

use gtk::prelude::*;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;

use self::model::WorkspaceSummary;
use crate::bar::BarMsg;
use crate::bar::state::{BarItemState, WorkspaceState};
use crate::bar::widget::{BarContext, BarWidget, BarWidgetRuntime, WidgetInstance};
use crate::shell::ShellMsg;

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
    ) -> Box<dyn BarWidgetRuntime> {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        render_status(&row, "Connecting to Niri");

        Box::new(WorkspacesRuntime { root: row })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Workspaces(WorkspaceState::Connecting))
    }

    fn start(&self, sender: relm4::Sender<ShellMsg>) -> Option<relm4::JoinHandle<()>> {
        Some(service::start_workspace_watcher(sender))
    }
}

pub(super) fn render_workspace_state(
    row: &gtk::Box,
    state: &WorkspaceState,
    monitor_connector: Option<&str>,
) {
    while let Some(child) = row.first_child() {
        row.remove(&child);
    }

    match state {
        WorkspaceState::Connecting => {
            render_status(row, "Connecting to Niri");
        }
        WorkspaceState::Ready(workspaces) => {
            render_workspaces(row, workspaces, monitor_connector);
        }
        WorkspaceState::Unavailable(error) => {
            render_status(row, &format!("Niri unavailable: {error}"));
        }
        WorkspaceState::UpdatesStopped => {
            render_status(row, "Niri updates stopped");
        }
    }
}

fn render_status(row: &gtk::Box, status: &str) {
    let label = gtk::Label::new(Some(status));
    label.add_css_class("status");
    row.append(&label);
}

fn render_workspaces(
    row: &gtk::Box,
    workspaces: &[WorkspaceSummary],
    monitor_connector: Option<&str>,
) {
    let workspaces = workspaces.iter().filter(|workspace| {
        let Some(monitor_connector) = monitor_connector else {
            return true;
        };

        workspace.output.as_deref() == Some(monitor_connector)
    });

    for workspace in workspaces {
        let label = gtk::Label::new(Some(&workspace.label()));

        for class_name in workspace.css_classes() {
            label.add_css_class(class_name);
        }

        row.append(&label);
    }
}
