use gtk::prelude::*;
use relm4::gtk;

use super::model::WorkspaceSummary;
use crate::bar::state::WorkspaceState;

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

pub(super) fn render_status(row: &gtk::Box, status: &str) {
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
