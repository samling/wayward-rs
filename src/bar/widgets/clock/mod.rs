mod component;
mod dropdown;

use chrono::Local;
use relm4::Controller;
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::prelude::*;
use std::time::Duration;

use crate::bar::state::{BarItemState, ClockState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetBuildContext, WidgetInstance,
};
use crate::shell::ShellMsg;

use self::component::{ClockComponent, ClockInit, ClockInput};

pub(crate) struct ClockWidget;

struct ClockRuntime {
    controller: Controller<ClockComponent>,
}

impl BarWidgetRuntime for ClockRuntime {
    fn root(&self) -> gtk::Widget {
        self.controller.widget().clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        let BarItemState::Clock(ClockState::Ready) = state else {
            return;
        };

        self.controller.emit(ClockInput::SetPlacement {
            edge: context.edge,
            region: context.region,
        });
        self.controller.emit(ClockInput::SetTime(Local::now()));
    }
}

impl BarWidget for ClockWidget {
    fn id(&self) -> &'static str {
        "clock"
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime> {
        let format = instance
            .config
            .get("format")
            .and_then(|value| value.as_str())
            .unwrap_or("%H:%M")
            .to_string();

        let controller = ClockComponent::builder()
            .launch(ClockInit {
                edge: context.bar.edge,
                region: context.bar.region,
                format,
                instance_class: instance.instance_css_class(),
            })
            .detach();

        Box::new(ClockRuntime { controller })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Clock(ClockState::Ready))
    }

    fn start(
        &self,
        sender: Sender<ShellMsg>,
        _services: &crate::services::ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        Some(start(sender))
    }
}

pub(crate) fn start(sender: Sender<ShellMsg>) -> relm4::tokio::task::JoinHandle<()> {
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
