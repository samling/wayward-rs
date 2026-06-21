mod component;
mod config;
mod dropdown;
mod service;
mod sunsetr;

use relm4::Controller;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::prelude::*;

use crate::bar::state::{BarItemState, BrightnessState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetBuildContext, WidgetEvent, WidgetInstance,
};
use crate::services::ShellServices;

use self::component::{BrightnessComponent, BrightnessInit, BrightnessInput};

pub(crate) struct BrightnessWidget;

pub(crate) static WIDGET: BrightnessWidget = BrightnessWidget;

struct BrightnessRuntime {
    controller: Controller<BrightnessComponent>,
}

impl BarWidgetRuntime for BrightnessRuntime {
    fn root(&self) -> gtk::Widget {
        self.controller.widget().clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        self.controller.emit(BrightnessInput::SetPlacement {
            edge: context.edge,
            region: context.region,
        });

        match state {
            BarItemState::Brightness(BrightnessState::Ready(snapshot)) => {
                self.controller
                    .emit(BrightnessInput::SetSnapshot(snapshot.clone()));
            }
            BarItemState::Brightness(BrightnessState::Unavailable(error)) => {
                self.controller
                    .emit(BrightnessInput::SetUnavailable(error.clone()));
            }
            _ => {}
        }
    }
}

impl BarWidget for BrightnessWidget {
    fn id(&self) -> &'static str {
        "brightness"
    }

    fn config_table_keys(&self) -> &'static [&'static str] {
        &["sunsetr"]
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime> {
        let config = instance.config_as::<config::BrightnessConfig>();

        let controller = BrightnessComponent::builder()
            .launch(BrightnessInit {
                edge: context.bar.edge,
                region: context.bar.region,
                bar_sender: context.sender.clone(),
                config,
            })
            .detach();

        Box::new(BrightnessRuntime { controller })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Brightness(BrightnessState::Unavailable(
            "Brightness has not loaded yet".to_string(),
        )))
    }

    fn handle_event(&self, event: WidgetEvent, services: &ShellServices) {
        service::handle_event(event, services.brightness.clone());
    }

    fn start(
        &self,
        sender: relm4::Sender<crate::shell::ShellMsg>,
        services: &ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        service::start(sender, services.brightness.clone())
    }
}
