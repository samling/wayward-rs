mod component;
mod dropdown;
mod service;

use relm4::Controller;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::prelude::*;

use crate::bar::state::BarItemState;
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetBuildContext, WidgetEvent, WidgetInstance,
};

use crate::services::ShellServices;

use self::component::{ActionMenuComponent, ActionMenuInit, ActionMenuInput};

pub(crate) struct ActionMenuWidget;

struct ActionMenuRuntime {
    controller: Controller<ActionMenuComponent>,
}

impl BarWidgetRuntime for ActionMenuRuntime {
    fn root(&self) -> gtk::Widget {
        self.controller.widget().clone().upcast()
    }

    fn update(&mut self, _state: &BarItemState, context: &BarContext) {
        self.controller.emit(ActionMenuInput::SetPlacement {
            edge: context.edge,
            region: context.region,
        });
    }
}

impl BarWidget for ActionMenuWidget {
    fn id(&self) -> &'static str {
        "action_menu"
    }

    fn build(
        &self,
        _instance: &WidgetInstance,
        context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime> {
        let controller = ActionMenuComponent::builder()
            .launch(ActionMenuInit {
                edge: context.bar.edge,
                region: context.bar.region,
                bar_sender: context.sender.clone(),
            })
            .detach();

        Box::new(ActionMenuRuntime { controller })
    }

    fn handle_event(&self, event: WidgetEvent, _services: &ShellServices) {
        service::handle_event(event);
    }
}