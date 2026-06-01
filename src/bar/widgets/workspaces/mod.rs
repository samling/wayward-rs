pub(crate) mod model;
pub(crate) mod service;

use gtk::prelude::*;
use relm4::gtk;

use self::model::WorkspaceSummary;
use crate::bar::state::{BarItemState, WorkspaceState};
use crate::bar::widget::{BarWidget, WidgetInstance};
use crate::bar::{Bar, BarMsg};
use crate::shell::ShellMsg;

pub(crate) struct WorkspacesWidget;

impl BarWidget for WorkspacesWidget {
    fn id(&self) -> &'static str {
        "workspaces"
    }

    fn render(
        &self,
        bar: &Bar,
        _instance: &WidgetInstance,
        container: &gtk::Box,
        _sender: &relm4::Sender<BarMsg>,
    ) {
        render_workspace_row(bar, container);
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Workspaces(WorkspaceState::Connecting))
    }

    fn start(&self, sender: relm4::Sender<ShellMsg>) -> Option<relm4::JoinHandle<()>> {
        Some(service::start_workspace_watcher(sender))
    }
}

pub(super) fn render_workspace_row(bar: &Bar, row: &gtk::Box) {
    let Some(state) = workspace_state(bar) else {
        render_status(row, "Connecting to Niri");
        return;
    };

    match state {
        WorkspaceState::Connecting => {
            render_status(row, "Connecting to Niri");
        }
        WorkspaceState::Ready(workspaces) => {
            render_workspaces(row, workspaces, bar.monitor_connector());
        }
        WorkspaceState::Unavailable(error) => {
            render_status(row, &format!("Niri unavailable: {error}"));
        }
        WorkspaceState::UpdatesStopped => {
            render_status(row, "Niri updates stopped");
        }
    }
}

fn workspace_state(bar: &Bar) -> Option<&WorkspaceState> {
    bar.item_states().iter().find_map(|state| match state {
        BarItemState::Workspaces(state) => Some(state),
        _ => None,
    })
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
