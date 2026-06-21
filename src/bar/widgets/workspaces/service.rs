use std::sync::Arc;

use futures::StreamExt;
use relm4::Sender;
use wayle_niri::{NiriService, WorkspaceReferenceArg};

use crate::bar::state::WorkspaceState;
use crate::bar::widget::{WidgetAction, WidgetEvent, WorkspaceAction};
use crate::bar::widgets::workspaces::model::WorkspaceSummary;
use crate::shell::ShellMsg;

pub fn start_workspace_watcher(
    sender: relm4::Sender<ShellMsg>,
    service: Option<Arc<NiriService>>,
) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_workspace_watcher(sender, service).await;
    })
}

pub async fn run_workspace_watcher(sender: Sender<ShellMsg>, service: Option<Arc<NiriService>>) {
    let Some(service) = service else {
        let _ = sender.send(workspace_message(WorkspaceState::Unavailable(
            "Niri service unavailable".to_string(),
        )));
        return;
    };

    let _ = send_workspace_snapshot(&sender, &service.as_ref());

    let mut events = service.events();
    while events.next().await.is_some() {
        if send_workspace_snapshot(&sender, &service.as_ref()).is_err() {
            return;
        }
    }

    let _ = sender.send(workspace_message(WorkspaceState::UpdatesStopped));
}

fn send_workspace_snapshot(sender: &Sender<ShellMsg>, service: &NiriService) -> Result<(), ()> {
    let mut summaries: Vec<_> = service
        .workspaces
        .get()
        .values()
        .map(|workspace| WorkspaceSummary::from_wayle_workspace(workspace.as_ref()))
        .collect();
    summaries.sort_by(|left, right| {
        left.output
            .cmp(&right.output)
            .then(left.idx.cmp(&right.idx))
            .then(left.id.cmp(&right.id))
    });

    sender
        .send(workspace_message(WorkspaceState::Ready(summaries)))
        .map_err(|_| ())
}

pub(crate) fn handle_event(event: WidgetEvent, service: Option<Arc<NiriService>>) {
    match event.action {
        WidgetAction::Workspaces(WorkspaceAction::Clicked {
            item_id, button, ..
        }) => {
            handle_click(item_id, button, service);
        }
        _ => {}
    }
}

fn clicked_workspace_reference(item_id: &str, button: u32) -> Option<WorkspaceReferenceArg> {
    if button != 1 {
        return None;
    }

    let Ok(workspace_id) = item_id.parse::<u64>() else {
        tracing::warn!("Ignoring workspace click with invalid id: {item_id}");
        return None;
    };

    Some(WorkspaceReferenceArg::Id(workspace_id))
}

fn handle_click(item_id: String, button: u32, service: Option<Arc<NiriService>>) {
    let Some(reference) = clicked_workspace_reference(&item_id, button) else {
        return;
    };

    let Some(service) = service else {
        tracing::warn!("Ignoring workspace click before service is ready");
        return;
    };

    relm4::spawn(async move {
        if let Err(error) = service.focus_workspace(reference).await {
            tracing::error!("Failed to focus workspace {item_id}: {error}");
        }
    });
}

fn workspace_message(state: WorkspaceState) -> ShellMsg {
    ShellMsg::ItemStateChanged(crate::bar::state::BarItemState::Workspaces(state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clicked_workspace_reference_uses_left_click_workspace_id() {
        assert_eq!(
            clicked_workspace_reference("42", 1),
            Some(WorkspaceReferenceArg::Id(42))
        );
    }

    #[test]
    fn clicked_workspace_reference_ignores_non_left_clicks() {
        assert_eq!(clicked_workspace_reference("42", 2), None);
        assert_eq!(clicked_workspace_reference("42", 3), None);
    }

    #[test]
    fn clicked_workspace_reference_ignores_invalid_workspace_ids() {
        assert_eq!(clicked_workspace_reference("not-a-workspace", 1), None);
    }
}
