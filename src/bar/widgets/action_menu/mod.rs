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
use crate::settings_spec::{
    NumberSpec, SettingSpec, SettingsSectionSpec, StringListSpec, StringSpec, table_string,
    table_string_list, table_u16,
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

    fn settings_sections(&self, config: &toml::value::Table) -> Vec<SettingsSectionSpec> {
        let sub = |key: &str| config.get(key).and_then(|value| value.as_table());
        let panel = sub("panel");
        let layout = sub("layout");
        let header = sub("header");

        vec![
            SettingsSectionSpec {
                title: "Panel".to_string(),
                settings: vec![SettingSpec::Number(NumberSpec {
                    label: "Width",
                    description: None,
                    path: &["widgets", "action_menu", "panel", "width"],
                    value: panel.and_then(|table| table_u16(table, "width")),
                    default: 268,
                    min: 120.0,
                    max: 1200.0,
                    step: 4.0,
                })],
            },
            SettingsSectionSpec {
                title: "Layout".to_string(),
                settings: vec![
                    SettingSpec::Number(NumberSpec {
                        label: "Button width",
                        description: None,
                        path: &["widgets", "action_menu", "layout", "button-width"],
                        value: layout.and_then(|table| table_u16(table, "button-width")),
                        default: 40,
                        min: 16.0,
                        max: 200.0,
                        step: 2.0,
                    }),
                    SettingSpec::Number(NumberSpec {
                        label: "Button height",
                        description: None,
                        path: &["widgets", "action_menu", "layout", "button-height"],
                        value: layout.and_then(|table| table_u16(table, "button-height")),
                        default: 40,
                        min: 16.0,
                        max: 200.0,
                        step: 2.0,
                    }),
                    SettingSpec::Number(NumberSpec {
                        label: "Row spacing",
                        description: None,
                        path: &["widgets", "action_menu", "layout", "row-spacing"],
                        value: layout.and_then(|table| table_u16(table, "row-spacing")),
                        default: 12,
                        min: 0.0,
                        max: 48.0,
                        step: 1.0,
                    }),
                ],
            },
            SettingsSectionSpec {
                title: "Header".to_string(),
                settings: vec![
                    SettingSpec::String(StringSpec {
                        label: "Power command",
                        description: None,
                        path: &["widgets", "action_menu", "header", "power-command"],
                        value: header.and_then(|table| table_string(table, &["power-command"])),
                        default: "wlogout",
                    }),
                    SettingSpec::StringList(StringListSpec {
                        label: "Power command args",
                        description: None,
                        path: &["widgets", "action_menu", "header", "power-args"],
                        value: header.and_then(|table| table_string_list(table, "power-args")),
                        default: &[],
                    }),
                ],
            },
        ]
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
