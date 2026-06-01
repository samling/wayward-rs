use chrono;
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::prelude::BoxExt;
use std::time::Duration;

use crate::bar::state::{BarItemState, ClockState};
use crate::bar::widget::{BarWidget, WidgetInstance};
use crate::bar::{Bar, BarMsg, style};
use crate::shell::ShellMsg;

pub(crate) struct ClockWidget;

impl BarWidget for ClockWidget {
    fn id(&self) -> &'static str {
        "clock"
    }

    fn render(
        &self,
        bar: &Bar,
        instance: &WidgetInstance,
        container: &gtk::Box,
        _sender: &relm4::Sender<BarMsg>,
    ) {
        let format = instance
            .config
            .get("format")
            .and_then(|value| value.as_str())
            .unwrap_or("%H:%M");

        let text = bar
            .item_states()
            .iter()
            .find_map(|state| match state {
                BarItemState::Clock(ClockState::Ready) => {
                    Some(chrono::Local::now().format(format).to_string())
                }
                _ => None,
            })
            .unwrap_or_else(|| chrono::Local::now().format(format).to_string());

        render(container, &text);
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Clock(ClockState::Ready))
    }

    fn start(&self, sender: relm4::Sender<ShellMsg>) -> Option<relm4::JoinHandle<()>> {
        Some(start(sender))
    }
}

pub(crate) fn start(sender: relm4::Sender<ShellMsg>) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_clock(sender).await;
    })
}

pub(super) async fn run_clock(sender: Sender<ShellMsg>) {
    loop {
        relm4::tokio::time::sleep(Duration::from_secs(1)).await;

        if sender.send(clock_message()).is_err() {
            return;
        }
    }
}

pub(super) fn render(container: &gtk::Box, text: &str) {
    let label = gtk::Label::new(Some(text));
    style::style_label(&label, "clock");
    container.append(&label);
}

fn clock_message() -> ShellMsg {
    ShellMsg::ItemStateChanged(BarItemState::Clock(ClockState::Ready))
}
