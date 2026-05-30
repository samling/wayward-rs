use chrono;
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::prelude::BoxExt;
use std::time::Duration;

use crate::bar::item;

use super::BarMsg;

pub(super) fn initial_text() -> String {
    current_time_text()
}

pub(super) fn current_time_text() -> String {
    chrono::Local::now().format("%H:%M").to_string()
}

pub(super) fn start(sender: relm4::Sender<BarMsg>) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_clock(sender).await;
    })
}

pub(super) async fn run_clock(sender: Sender<BarMsg>) {
    loop {
        relm4::tokio::time::sleep(Duration::from_secs(60)).await;

        if sender
            .send(BarMsg::ClockChanged(current_time_text()))
            .is_err()
        {
            return;
        }
    }
}

pub(super) fn render(container: &gtk::Box, text: &str) {
    let label = gtk::Label::new(Some(text));
    item::style_label(&label, "clock");
    container.append(&label);
}
