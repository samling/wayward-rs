use std::process::{Output, Stdio};
use std::sync::OnceLock;
use std::time::Duration;

use relm4::Sender;
use relm4::tokio::process::Command;
use relm4::tokio::sync::broadcast;
use relm4::tokio::time;

use serde::Deserialize;

use crate::bar::BarMsg;
use crate::bar::state::{BarItemState, UpdatesState};

use super::model::{
    UpdateSeverityMatcher, UpdatesSnapshot, parse_checkupdates_output, sort_packages_by_severity,
};

static REFRESH_REQUESTS: OnceLock<broadcast::Sender<()>> = OnceLock::new();

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct UpdatesServiceConfig {
    pub(super) refresh_interval_minutes: u64,
    pub(super) critical_patterns: Vec<String>,
    pub(super) warning_patterns: Vec<String>,
}

impl Default for UpdatesServiceConfig {
    fn default() -> Self {
        Self {
            refresh_interval_minutes: 30,
            critical_patterns: Vec::new(),
            warning_patterns: Vec::new(),
        }
    }
}

pub(super) fn request_refresh() {
    if let Some(sender) = REFRESH_REQUESTS.get() {
        let _ = sender.send(());
    }
}

pub(super) fn start(
    sender: Sender<BarMsg>,
    config: UpdatesServiceConfig,
) -> relm4::tokio::task::JoinHandle<()> {
    let refresh_sender = REFRESH_REQUESTS
        .get_or_init(|| {
            let (sender, _) = broadcast::channel(8);
            sender
        })
        .clone();

    relm4::spawn(async move {
        run_updates_watcher(sender, config, refresh_sender.subscribe()).await;
    })
}

async fn run_updates_watcher(
    sender: Sender<BarMsg>,
    config: UpdatesServiceConfig,
    mut refresh_requests: broadcast::Receiver<()>,
) {
    let mut latest_snapshot = None;

    refresh_updates(&sender, &config, &mut latest_snapshot).await;

    let mut interval = time::interval(Duration::from_secs(
        config.refresh_interval_minutes.max(1) * 60,
    ));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                refresh_updates(&sender, &config, &mut latest_snapshot).await;
            }
            request = refresh_requests.recv() => {
                match request {
                    Ok(()) => refresh_updates(&sender, &config, &mut latest_snapshot).await,
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        refresh_updates(&sender, &config, &mut latest_snapshot).await;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

async fn refresh_updates(
    sender: &Sender<BarMsg>,
    config: &UpdatesServiceConfig,
    latest_snapshot: &mut Option<UpdatesSnapshot>,
) {
    let _ = sender.send(updates_message(UpdatesState::Ready(refreshing_snapshot(
        latest_snapshot.as_ref(),
    ))));

    match load_updates(config).await {
        Ok(snapshot) => {
            *latest_snapshot = Some(snapshot.clone());
            let _ = sender.send(updates_message(UpdatesState::Ready(snapshot)));
        }
        Err(error) => {
            tracing::warn!(?error, "Failed to check package updates");

            if let Some(snapshot) = latest_snapshot.as_ref() {
                let mut snapshot = snapshot.clone();
                snapshot.last_error = Some(error);
                snapshot.refreshing = false;
                let _ = sender.send(updates_message(UpdatesState::Ready(snapshot)));
            } else {
                let _ = sender.send(updates_message(UpdatesState::Unavailable(error)));
            }
        }
    }
}

async fn load_updates(config: &UpdatesServiceConfig) -> Result<UpdatesSnapshot, String> {
    let output = Command::new("checkupdates")
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|error| format!("failed to run checkupdates: {error}"))?;

    if !output.status.success() && output.status.code() != Some(2) {
        return Err(format!(
            "checkupdates failed ({}): {}",
            output.status,
            command_output_message(&output)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = parse_checkupdates_output(&stdout);

    let matcher = UpdateSeverityMatcher::new(&config.critical_patterns, &config.warning_patterns)
        .map_err(|error| format!("invalid update severity pattern: {error}"))?;

    matcher.apply(&mut packages);
    sort_packages_by_severity(&mut packages);

    Ok(UpdatesSnapshot {
        packages,
        last_error: None,
        refreshing: false,
    })
}

fn updates_message(state: UpdatesState) -> BarMsg {
    BarMsg::ItemStateChanged(BarItemState::Updates(state))
}

fn refreshing_snapshot(latest_snapshot: Option<&UpdatesSnapshot>) -> UpdatesSnapshot {
    let mut snapshot = latest_snapshot.cloned().unwrap_or_else(|| UpdatesSnapshot {
        packages: Vec::new(),
        last_error: None,
        refreshing: false,
    });

    snapshot.refreshing = true;
    snapshot
}

fn command_output_message(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }

    "no output".to_string()
}

#[cfg(test)]
mod tests {
    use super::super::model::{UpdatePackage, UpdateSeverity};
    use super::*;

    #[test]
    fn refreshing_snapshot_preserves_latest_packages() {
        let latest = UpdatesSnapshot {
            packages: vec![UpdatePackage {
                name: "linux".to_string(),
                old_version: "6.9.1.arch1-1".to_string(),
                new_version: "6.9.2.arch1-1".to_string(),
                severity: UpdateSeverity::Critical,
            }],
            last_error: None,
            refreshing: false,
        };

        let snapshot = refreshing_snapshot(Some(&latest));

        assert!(snapshot.refreshing);
        assert_eq!(snapshot.packages, latest.packages);
    }
}
