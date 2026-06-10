mod component;
mod config;
mod icon;
mod interaction;
pub(crate) mod service;
pub(crate) mod view_model;

pub(crate) const ID: &str = "systray";

use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::{Component, ComponentController, Controller, Sender};

use self::component::{SystrayComponent, SystrayInit, SystrayInput};
use self::config::SystrayConfig;
use crate::bar::state::{BarItemState, SystrayState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetBuildContext, WidgetEvent, WidgetInstance,
};
use crate::shell::ShellMsg;

struct SystrayRuntime {
    controller: Controller<SystrayComponent>,
}

impl BarWidgetRuntime for SystrayRuntime {
    fn root(&self) -> gtk::Widget {
        self.controller.widget().clone().upcast()
    }

    fn set_context(&mut self, context: &BarContext) {
        self.controller
            .emit(SystrayInput::SetOrientation(context.edge.orientation()));
    }

    fn update(&mut self, state: &BarItemState, _context: &BarContext) {
        let BarItemState::Systray(SystrayState::Ready(items)) = state else {
            return;
        };

        self.controller.emit(SystrayInput::SetItems(items.clone()));
    }
}

pub(crate) struct SystrayWidget;

impl BarWidget for SystrayWidget {
    fn id(&self) -> &'static str {
        ID
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime> {
        let config = instance.config_as::<SystrayConfig>();
        let controller = SystrayComponent::builder()
            .launch(SystrayInit {
                edge: context.bar.edge,
                icon_size: config.icon_size(),
                instance_class: instance.instance_css_class(),
                sender: context.sender.clone(),
                service: context.services.systray.clone(),
            })
            .detach();

        Box::new(SystrayRuntime { controller })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Systray(SystrayState::Ready(Vec::new())))
    }

    fn handle_event(&self, event: WidgetEvent, services: &crate::services::ShellServices) {
        service::handle_event(event, services.systray.clone());
    }

    fn start(
        &self,
        sender: Sender<ShellMsg>,
        services: &crate::services::ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        Some(service::start(sender, services.systray.clone()))
    }
}
