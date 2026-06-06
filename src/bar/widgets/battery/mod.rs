mod component;
mod dropdown;
mod format;
mod service;
mod view_model;

use crate::bar::state::{BarItemState, BatteryState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetBuildContext, WidgetInstance,
};
use crate::shell::ShellMsg;
use relm4::Controller;
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::prelude::*;

use self::component::{BatteryComponent, BatteryInit, BatteryInput};

struct BatteryRuntime {
    controller: Controller<BatteryComponent>,
}

impl BarWidgetRuntime for BatteryRuntime {
    fn root(&self) -> gtk::Widget {
        self.controller.widget().clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        self.controller.emit(BatteryInput::SetEdge(context.edge));

        match state {
            BarItemState::Battery(BatteryState::Ready(snapshot)) => {
                self.controller
                    .emit(BatteryInput::SetSnapshot(snapshot.clone()));
            }

            BarItemState::Battery(BatteryState::Unavailable) => {
                self.controller.emit(BatteryInput::SetUnavailable);
            }
            _ => {}
        }
    }
}

pub(crate) struct BatteryWidget;

impl BarWidget for BatteryWidget {
    fn id(&self) -> &'static str {
        "battery"
    }

    fn build(
        &self,
        _instance: &WidgetInstance,
        context: &WidgetBuildContext,
    ) -> Box<dyn BarWidgetRuntime> {
        let controller = BatteryComponent::builder()
            .launch(BatteryInit {
                edge: context.bar.edge,
                power_profiles: context.services.power_profiles.clone(),
            })
            .detach();

        Box::new(BatteryRuntime { controller })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Battery(BatteryState::Unavailable))
    }

    fn start(
        &self,
        sender: Sender<ShellMsg>,
        services: &crate::services::ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        Some(service::start(
            sender,
            services.battery.clone(),
            services.power_profiles.clone(),
        ))
    }
}
