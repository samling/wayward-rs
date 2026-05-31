use futures::StreamExt;
use relm4::Sender;
use wayle_niri::NiriService;

use crate::bar::state::WorkspaceState;
use crate::shell::ShellMsg;
use crate::workspace::WorkspaceSummary;

pub fn start_workspace_watcher(
    sender: relm4::Sender<ShellMsg>,
) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_workspace_watcher(sender).await;
    })
}

pub async fn run_workspace_watcher(sender: Sender<ShellMsg>) {
    let service = match NiriService::new().await {
        Ok(service) => service,
        Err(error) => {
            let _ = sender.send(workspace_message(WorkspaceState::Unavailable(
                error.to_string(),
            )));
            return;
        }
    };

    let _ = send_workspace_snapshot(&sender, &service);

    let mut events = service.events();
    while events.next().await.is_some() {
        if send_workspace_snapshot(&sender, &service).is_err() {
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
