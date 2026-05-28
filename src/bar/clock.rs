use std::time::Duration;
use relm4::Sender;
use chrono;

use super::BarMsg;

pub(super) fn current_time_text() -> String {
    chrono::Local::now().format("%H:%M").to_string()
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