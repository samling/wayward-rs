mod component;
mod config;
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
use self::config::ActionMenuConfig;

pub(crate) struct ActionMenuWidget;

pub(crate) static WIDGET: ActionMenuWidget = ActionMenuWidget;

/// The built-in default sections, as TOML values. Used to seed the config file
/// with defaults on first run (when it has no `sections`), keeping the config the
/// single source of truth for the settings editor.
pub(crate) fn default_sections() -> toml::value::Array {
    ActionMenuConfig::default()
        .sections
        .iter()
        .filter_map(|section| toml::Value::try_from(section).ok())
        .collect()
}

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

    fn config_table_keys(&self) -> &'static [&'static str] {
        &["panel", "layout", "header"]
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime> {
        let config = instance.config_as::<ActionMenuConfig>();
        let controller = ActionMenuComponent::builder()
            .launch(ActionMenuInit {
                edge: context.bar.edge,
                region: context.bar.region,
                bar_sender: context.sender.clone(),
                config,
            })
            .detach();

        Box::new(ActionMenuRuntime { controller })
    }

    fn handle_event(&self, event: WidgetEvent, _services: &ShellServices) {
        service::handle_event(event);
    }
}
