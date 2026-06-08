use std::process::Command;

use crate::bar::widget::{ActionMenuAction, WidgetAction, WidgetEvent};

pub(super) fn handle_event(event: WidgetEvent) {
    match event.action {
        WidgetAction::RunActionMenuAction { action } => handle_action(action),
        _ => {}
    }
}

fn handle_action(action: ActionMenuAction) {
    match action {
        ActionMenuAction::PowerMenu => run_command("wlogout", &[]),
        ActionMenuAction::ScreenshotRegion => run_command("screenshot", &["region"]),
        ActionMenuAction::ScreenshotWindow => run_command("screenshot", &["window"]),
        ActionMenuAction::ScreenshotScreen => run_command("screenshot", &["monitor"]),
    }
}

fn run_command(program: &str, args: &[&str]) {
    if let Err(error) = Command::new(program).args(args).spawn() {
        tracing::error!("Failed to run {program}: {error}");
    }
}