mod component;

use relm4::Controller;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::prelude::*;

use crate::bar::state::{BarItemState, NotificationState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetBuildContext, WidgetInstance,
};

use self::component::{NotificationsComponent, NotificationsInput};

pub(crate) struct NotificationsWidget;

struct NotificationsRuntime {
    controller: Controller<NotificationsComponent>,
}

impl BarWidgetRuntime for NotificationsRuntime {
    fn root(&self) -> gtk::Widget {
        self.controller.widget().clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, _context: &BarContext) {
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
        _context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime> {
        let controller = NotificationsComponent::builder().launch(()).detach();

        Box::new(NotificationsRuntime { controller })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Notifications(NotificationState::Unavailable))
    }
}