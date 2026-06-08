use std::process::Command;

use crate::bar::widget::{ActionMenuCommand, WidgetAction, WidgetEvent};

pub(super) fn handle_event(event: WidgetEvent) {
    match event.action {
        WidgetAction::RunActionMenuAction { command } => handle_command(command),
        _ => {}
    }
}

fn handle_command(command: ActionMenuCommand) {
    run_command(&command.program, &command.args);
}

fn run_command(program: &str, args: &[String]) {
    if let Err(error) = Command::new(program).args(args).spawn() {
        tracing::error!("Failed to run {program}: {error}");
    }
}