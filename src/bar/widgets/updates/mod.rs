mod component;
mod dropdown;
pub(crate) mod model;
mod row;
mod service;

use relm4::Controller;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::prelude::*;

use crate::bar::state::{BarItemState, UpdatesState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, UpdatesAction, WidgetAction, WidgetBuildContext,
    WidgetEvent, WidgetInstance,
};
use crate::services::ShellServices;
use crate::settings_spec::{SettingSpec, SettingsSectionSpec, StringListSpec, table_string_list};

use self::component::{UpdatesComponent, UpdatesInit, UpdatesInput};

pub(crate) struct UpdatesWidget;

pub(crate) static WIDGET: UpdatesWidget = UpdatesWidget;

struct UpdatesRuntime {
    controller: Controller<UpdatesComponent>,
}

impl BarWidgetRuntime for UpdatesRuntime {
    fn root(&self) -> gtk::Widget {
        self.controller.widget().clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        self.controller.emit(UpdatesInput::SetPlacement {
            edge: context.edge,
            region: context.region,
        });

        match state {
            BarItemState::Updates(UpdatesState::Ready(snapshot)) => {
                self.controller
                    .emit(UpdatesInput::SetSnapshot(snapshot.clone()));
            }
            BarItemState::Updates(UpdatesState::Unavailable(error)) => {
                self.controller
                    .emit(UpdatesInput::SetUnavailable(error.clone()));
            }
            _ => {}
        }
    }
}

impl BarWidget for UpdatesWidget {
    fn id(&self) -> &'static str {
        "updates"
    }

    fn settings_sections(
        &self,
        config: &toml::value::Table,
    ) -> Vec<crate::settings_spec::SettingsSectionSpec> {
        vec![SettingsSectionSpec {
            title: "Config".to_string(),
            settings: vec![
                SettingSpec::StringList(StringListSpec {
                    label: "Critical patterns",
                    description: None,
                    path: &["widgets", "updates", "critical-pattens"],
                    value: table_string_list(config, "critical-patterns"),
                    default: &[],
                }),
                SettingSpec::StringList(StringListSpec {
                    label: "Warning patterns",
                    description: None,
                    path: &["widgets", "updates", "warning-pattens"],
                    value: table_string_list(config, "warning-patterns"),
                    default: &[],
                }),
            ],
        }]
    }

    fn build(
        &self,
        _instance: &WidgetInstance,
        context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime> {
        let config = _instance.config_as::<self::service::UpdatesServiceConfig>();

        let controller = UpdatesComponent::builder()
            .launch(UpdatesInit {
                edge: context.bar.edge,
                region: context.bar.region,
                bar_sender: context.sender.clone(),
                config,
            })
            .detach();

        Box::new(UpdatesRuntime { controller })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Updates(UpdatesState::Unavailable(
            "Updates have not loaded yet".to_string(),
        )))
    }

    fn handle_event(&self, event: WidgetEvent, _services: &ShellServices) {
        match event.action {
            WidgetAction::Updates(UpdatesAction::Refresh) => {
                service::request_refresh();
            }
            _ => {}
        }
    }
}
