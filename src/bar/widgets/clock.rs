use chrono;
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use std::time::Duration;

use crate::bar::state::{BarItemState, ClockState};
use crate::bar::widget::BarWidgetRuntime;
use crate::bar::widget::{BarWidget, WidgetInstance};
use crate::bar::{BarContext, BarMsg, style};
use crate::shell::ShellMsg;

pub(crate) struct ClockWidget;

struct ClockRuntime {
    root: gtk::Label,
    format: String,
}

impl BarWidgetRuntime for ClockRuntime {
    fn root(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, _context: &BarContext) {
        let BarItemState::Clock(ClockState::Ready) = state else {
            return;
        };

        self.root
            .set_text(&chrono::Local::now().format(&self.format).to_string());
    }
}

impl BarWidget for ClockWidget {
    fn id(&self) -> &'static str {
        "clock"
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        _sender: &relm4::Sender<BarMsg>,
    ) -> Box<dyn BarWidgetRuntime> {
        let format = instance
            .config
            .get("format")
            .and_then(|value| value.as_str())
            .unwrap_or("%H:%M")
            .to_string();

        let label = gtk::Label::new(Some(&chrono::Local::now().format(&format).to_string()));

        style::style_label(&label, "clock");

        Box::new(ClockRuntime {
            root: label,
            format,
        })
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

fn clock_message() -> ShellMsg {
    ShellMsg::ItemStateChanged(BarItemState::Clock(ClockState::Ready))
}
