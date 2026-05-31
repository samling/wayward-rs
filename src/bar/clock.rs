use chrono;
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::prelude::BoxExt;
use std::time::Duration;

use crate::bar::item;
use crate::bar::state::{BarItemState, ClockState};
use crate::shell::ShellMsg;

pub(crate) fn initial_text() -> String {
    current_time_text()
}

pub(super) fn current_time_text() -> String {
    chrono::Local::now().format("%H:%M").to_string()
}

pub(crate) fn start(sender: relm4::Sender<ShellMsg>) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_clock(sender).await;
    })
}

pub(super) async fn run_clock(sender: Sender<ShellMsg>) {
    loop {
        relm4::tokio::time::sleep(Duration::from_secs(60)).await;

        if sender.send(clock_message(current_time_text())).is_err() {
            return;
        }
    }
}

pub(super) fn render(container: &gtk::Box, text: &str) {
    let label = gtk::Label::new(Some(text));
    item::style_label(&label, "clock");
    container.append(&label);
}

fn clock_message(text: String) -> ShellMsg {
    ShellMsg::ItemStateChanged(BarItemState::Clock(ClockState::Ready(text)))
}
