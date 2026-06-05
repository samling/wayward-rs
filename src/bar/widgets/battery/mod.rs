mod format;
mod service;
mod view;
mod view_model;

use crate::bar::state::{BarItemState, BatteryState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetBuildContext, WidgetInstance,
};
use crate::shell::ShellMsg;
use relm4::Sender;
use relm4::gtk;
use self::view::BatteryView;

struct BatteryRuntime {
    view: BatteryView,
}

impl BarWidgetRuntime for BatteryRuntime {
    fn root(&self) -> gtk::Widget {
        self.view.root()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        self.view.set_edge(context.edge);

        match state {
            BarItemState::Battery(BatteryState::Ready(snapshot)) => {
                self.view.show_snapshot(snapshot);
            }

            BarItemState::Battery(BatteryState::Unavailable) => {
                self.view.show_unavailable();
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
        instance: &WidgetInstance,
        context: &WidgetBuildContext,
    ) -> Box<dyn BarWidgetRuntime> {
        let view = BatteryView::new(instance, context);

        Box::new(BatteryRuntime { view })
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
