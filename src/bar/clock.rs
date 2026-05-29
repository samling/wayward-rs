use std::time::Duration;
use relm4::{Sender, gtk::prelude::WidgetExt};
use chrono;
use relm4::gtk;

use super::BarMsg;

pub(super) fn initial_text() -> String {
    current_time_text()
}

pub(super) fn current_time_text() -> String {
    chrono::Local::now().format("%H:%M").to_string()
}

pub(super) fn start(sender: relm4::Sender<BarMsg>) {
    relm4::spawn(async move {
        run_clock(sender).await;
    });
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

pub(super) fn render(label: &gtk::Label) {
    label.add_css_class("bar-item");
    label.add_css_class("clock");
}