use notify::{
    RecursiveMode, Watcher,
    event::{CreateKind, EventKind, ModifyKind},
};
use std::{path::PathBuf, sync::mpsc, thread, time::Duration};

pub(crate) fn start_debounced_file_watch<F>(
    label: &'static str,
    dir: PathBuf,
    target_path: PathBuf,
    on_change: F,
) where
    F: Fn() + Send + 'static,
{
    thread::spawn(move || {
        let (reload_tx, reload_rx) = mpsc::channel::<()>();
        let callback_tx = reload_tx.clone();

        let mut watcher = match notify::recommended_watcher(move |event| {
            if is_write_event_for_path(&event, &target_path) {
                let _ = callback_tx.send(());
            }
        }) {
            Ok(watcher) => watcher,
            Err(error) => {
                tracing::error!("Failed to create {label} watcher: {error}");
                return;
            }
        };

        if let Err(error) = watcher.watch(&dir, RecursiveMode::NonRecursive) {
            tracing::error!(
                "Failed to watch {label} directory {}: {error}",
                dir.display()
            );
            return;
        }

        tracing::info!("Watching {label} directory {}", dir.display());

        while reload_rx.recv().is_ok() {
            while reload_rx.recv_timeout(Duration::from_millis(150)).is_ok() {}

            on_change();
        }
    });
}

fn is_write_event_for_path(event: &notify::Result<notify::Event>, target_path: &PathBuf) -> bool {
    let Ok(event) = event else {
        tracing::error!("File watcher error: {event:?}");
        return false;
    };

    let touches_target = event.paths.iter().any(|path| path == target_path);

    if !touches_target {
        return false;
    }

    matches!(
        event.kind,
        EventKind::Create(CreateKind::File) | EventKind::Modify(ModifyKind::Data(_))
    )
}
