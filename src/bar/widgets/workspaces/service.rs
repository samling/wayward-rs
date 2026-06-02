use std::sync::Arc;

use futures::StreamExt;
use relm4::Sender;
use wayle_niri::NiriService;

use crate::bar::state::WorkspaceState;
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

fn workspace_message(state: WorkspaceState) -> ShellMsg {
    ShellMsg::ItemStateChanged(crate::bar::state::BarItemState::Workspaces(state))
}
