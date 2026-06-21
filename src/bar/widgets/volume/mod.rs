mod component;
mod dropdown;
pub(crate) mod model;
mod service;

use relm4::Controller;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::prelude::*;

use crate::bar::state::{BarItemState, VolumeState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetBuildContext, WidgetEvent, WidgetInstance,
};
use crate::services::ShellServices;

use self::component::{VolumeComponent, VolumeInit, VolumeInput};

pub(crate) struct VolumeWidget;

pub(crate) static WIDGET: VolumeWidget = VolumeWidget;

struct VolumeRuntime {
    controller: Controller<VolumeComponent>,
}

impl BarWidgetRuntime for VolumeRuntime {
    fn root(&self) -> gtk::Widget {
        self.controller.widget().clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        self.controller.emit(VolumeInput::SetPlacement {
            edge: context.edge,
            region: context.region,
        });

        match state {
            BarItemState::Volume(VolumeState::Ready(snapshot)) => {
                self.controller
                    .emit(VolumeInput::SetSnapshot(snapshot.clone()));
            }
            BarItemState::Volume(VolumeState::Unavailable(error)) => {
                self.controller
                    .emit(VolumeInput::SetUnavailable(error.clone()));
            }
            _ => {}
        }
    }
}

impl BarWidget for VolumeWidget {
    fn id(&self) -> &'static str {
        "volume"
    }

    fn build(
        &self,
        _instance: &WidgetInstance,
        context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime> {
        let controller = VolumeComponent::builder()
            .launch(VolumeInit {
                edge: context.bar.edge,
                region: context.bar.region,
                bar_sender: context.sender.clone(),
            })
            .detach();

        Box::new(VolumeRuntime { controller })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Volume(VolumeState::Unavailable(
            "Volume has not loaded yet".to_string(),
        )))
    }

    fn handle_event(&self, event: WidgetEvent, services: &ShellServices) {
        service::handle_event(event, services.audio.clone());
    }

    fn start(
        &self,
        sender: relm4::Sender<crate::shell::ShellMsg>,
        services: &ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        service::start(sender, services.audio.clone())
    }
}
