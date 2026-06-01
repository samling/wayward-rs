use chrono;
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::prelude::BoxExt;
use std::time::Duration;

use crate::bar::state::{BarItemState, ClockState};
use crate::bar::widget::BarWidget;
use crate::bar::{Bar, style};
use crate::shell::ShellMsg;

pub(crate) struct ClockWidget;

impl BarWidget for ClockWidget {
    fn id(&self) -> &'static str {
        "clock"
    }

    fn render(&self, bar: &Bar, container: &gtk::Box) {
        let fallback = initial_text();

        let text = bar
            .item_states()
            .iter()
            .find_map(|state| match state {
                BarItemState::Clock(ClockState::Ready(text)) => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or(fallback.as_str());

        render(container, text);
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Clock(ClockState::Ready(initial_text())))
    }

    fn start(&self, sender: relm4::Sender<ShellMsg>) -> Option<relm4::JoinHandle<()>> {
        Some(start(sender))
    }
}

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
    style::style_label(&label, "clock");
    container.append(&label);
}

fn clock_message(text: String) -> ShellMsg {
    ShellMsg::ItemStateChanged(BarItemState::Clock(ClockState::Ready(text)))
}
