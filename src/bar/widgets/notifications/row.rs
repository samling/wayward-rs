use relm4::factory::FactoryComponent;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use std::rc::Rc;

use crate::notifications::card::{
    NotificationCardCallbacks, dropdown_card_options, notification_card,
};
use crate::notifications::model::NotificationToast;

pub(super) struct NotificationRow {
    notification: NotificationToast,
}

#[derive(Debug)]
pub(super) enum NotificationRowInput {
    InvokeDefault,
    InvokeAction(String),
    Dismiss,
}

#[derive(Debug)]
pub(super) enum NotificationRowOutput {
    InvokeDefault(u32),
    InvokeAction { id: u32, action_id: String },
    Dismiss(u32),
}

#[relm4::factory(pub(super))]
impl FactoryComponent for NotificationRow {
    type Init = NotificationToast;
    type Input = NotificationRowInput;
    type Output = NotificationRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[name = "root"]
        gtk::Box {
            add_css_class: "notification-list-row-wrapper",
            set_orientation: gtk::Orientation::Vertical,
        }
    }

    fn pre_view() {
        self.sync_row_widgets(&widgets.root, sender.clone());
    }

    fn init_model(
        notification: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { notification }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        self.sync_row_widgets(&widgets.root, sender);
        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            NotificationRowInput::InvokeDefault => {
                let _ = sender.output(NotificationRowOutput::InvokeDefault(self.notification.id));
            }
            NotificationRowInput::InvokeAction(action_id) => {
                let _ = sender.output(NotificationRowOutput::InvokeAction {
                    id: self.notification.id,
                    action_id,
                });
            }
            NotificationRowInput::Dismiss => {
                let _ = sender.output(NotificationRowOutput::Dismiss(self.notification.id));
            }
        }
    }
}

impl NotificationRow {
    fn sync_row_widgets(&self, root: &gtk::Box, sender: FactorySender<Self>) {
        while let Some(child) = root.first_child() {
            root.remove(&child);
        }

        let default_sender = sender.clone();
        let action_sender = sender.clone();
        let dismiss_sender = sender;

        let card = notification_card(
            &self.notification,
            dropdown_card_options(),
            NotificationCardCallbacks {
                on_default: Some(Rc::new(move |_| {
                    default_sender.input(NotificationRowInput::InvokeDefault);
                })),
                on_action: Some(Rc::new(move |_, action_id| {
                    action_sender.input(NotificationRowInput::InvokeAction(action_id));
                })),
                on_dismiss: Some(Rc::new(move |_| {
                    dismiss_sender.input(NotificationRowInput::Dismiss);
                })),
            },
        );

        root.append(&card);
    }

    pub(super) fn set_notification(&mut self, notification: NotificationToast) {
        self.notification = notification;
    }
}
