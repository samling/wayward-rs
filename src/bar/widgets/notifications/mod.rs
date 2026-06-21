mod component;
mod dropdown;
mod row;
mod service;

use relm4::Controller;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::prelude::*;

use crate::bar::state::{BarItemState, NotificationState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetBuildContext, WidgetEvent, WidgetInstance,
};
use crate::services::ShellServices;

use self::component::{NotificationsComponent, NotificationsInit, NotificationsInput};

pub(crate) struct NotificationsWidget;

pub(crate) static WIDGET: NotificationsWidget = NotificationsWidget;

struct NotificationsRuntime {
    controller: Controller<NotificationsComponent>,
}

impl BarWidgetRuntime for NotificationsRuntime {
    fn root(&self) -> gtk::Widget {
        self.controller.widget().clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        self.controller.emit(NotificationsInput::SetPlacement {
            edge: context.edge,
            region: context.region,
        });
        match state {
            BarItemState::Notifications(NotificationState::Ready(notifications)) => {
                self.controller
                    .emit(NotificationsInput::SetNotifications(notifications.clone()));
            }
            BarItemState::Notifications(NotificationState::Unavailable) => {
                self.controller.emit(NotificationsInput::SetUnavailable);
            }
            _ => {}
        }
    }
}

impl BarWidget for NotificationsWidget {
    fn id(&self) -> &'static str {
        "notifications"
    }

    fn build(
        &self,
        _instance: &WidgetInstance,
        context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime> {
        let controller = NotificationsComponent::builder()
            .launch(NotificationsInit {
                edge: context.bar.edge,
                region: context.bar.region,
                bar_sender: context.sender.clone(),
            })
            .detach();

        Box::new(NotificationsRuntime { controller })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Notifications(NotificationState::Unavailable))
    }

    fn handle_event(&self, event: WidgetEvent, services: &ShellServices) {
        service::handle_event(event, services.notification.clone());
    }
}
