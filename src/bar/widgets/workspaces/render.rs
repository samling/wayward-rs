use gtk::prelude::*;
use relm4::gtk;

use super::model::WorkspaceSummary;
use crate::bar::state::WorkspaceState;

pub(super) struct RenderedWorkspace {
    id: u64,
    root: gtk::Box,
    label: gtk::Label,
}

pub(super) fn render_workspace_state(
    row: &gtk::Box,
    rendered: &mut Vec<RenderedWorkspace>,
    state: &WorkspaceState,
    monitor_connector: Option<&str>,
    label_format: &str,
) -> Option<gtk::Box> {
    match state {
        WorkspaceState::Connecting => {
            clear(row, rendered);
            render_status(row, "Connecting to Niri");
            None
        }
        WorkspaceState::Ready(workspaces) => {
            render_workspaces(row, rendered, workspaces, monitor_connector, label_format)
        }
        WorkspaceState::Unavailable(error) => {
            clear(row, rendered);
            render_status(row, &format!("Niri unavailable: {error}"));
            None
        }
        WorkspaceState::UpdatesStopped => {
            clear(row, rendered);
            render_status(row, "Niri updates stopped");
            None
        }
    }
}

fn clear(row: &gtk::Box, rendered: &mut Vec<RenderedWorkspace>) {
    while let Some(child) = row.first_child() {
        row.remove(&child);
    }

    rendered.clear();
}

pub(super) fn render_status(row: &gtk::Box, status: &str) {
    let label = gtk::Label::new(Some(status));
    label.add_css_class("status");
    row.append(&label);
}

fn render_workspaces(
    row: &gtk::Box,
    rendered: &mut Vec<RenderedWorkspace>,
    workspaces: &[WorkspaceSummary],
    monitor_connector: Option<&str>,
    label_format: &str,
) -> Option<gtk::Box> {
    let visible: Vec<_> = workspaces
        .iter()
        .filter(|workspace| {
            let Some(monitor_connector) = monitor_connector else {
                return true;
            };

            workspace.output.as_deref() == Some(monitor_connector)
        })
        .collect();

    if rendered.is_empty() {
        while let Some(child) = row.first_child() {
            row.remove(&child);
        }
    }

    rendered.retain(|rendered_workspace| {
        let keep = visible
            .iter()
            .any(|workspace| workspace.id == rendered_workspace.id);

        if !keep {
            row.remove(&rendered_workspace.root);
        }

        keep
    });

    let mut active_label = None;

    for workspace in visible {
        let (root, label) = match rendered
            .iter()
            .find(|rendered_workspace| rendered_workspace.id == workspace.id)
        {
            Some(rendered_workspace) => (
                rendered_workspace.root.clone(),
                rendered_workspace.label.clone(),
            ),
            None => {
                let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                root.add_css_class("workspace");
                root.set_halign(gtk::Align::Center);
                root.set_valign(gtk::Align::Center);

                let label = gtk::Label::new(None);
                label.add_css_class("workspace-label");
                label.set_hexpand(true);
                label.set_halign(gtk::Align::Center);
                label.set_xalign(0.5);

                root.append(&label);
                row.append(&root);

                rendered.push(RenderedWorkspace {
                    id: workspace.id,
                    root: root.clone(),
                    label: label.clone(),
                });

                (root, label)
            }
        };

        label.set_text(&workspace.formatted_label(label_format));
        apply_workspace_classes(&root, workspace);

        if workspace.is_active {
            active_label = Some(root);
        }
    }

    active_label
}

fn apply_workspace_classes(root: &gtk::Box, workspace: &WorkspaceSummary) {
    for class_name in ["active", "focused", "urgent"] {
        root.remove_css_class(class_name);
    }

    if workspace.is_active {
        root.add_css_class("active");
    }

    if workspace.is_focused {
        root.add_css_class("focused");
    }

    if workspace.is_urgent {
        root.add_css_class("urgent");
    }
}
