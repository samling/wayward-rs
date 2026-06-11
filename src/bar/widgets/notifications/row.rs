use relm4::factory::FactoryComponent;
use relm4::gtk::prelude::*;
use relm4::gtk::{self, glib};
use relm4::prelude::*;
use std::time::Duration;

use crate::notifications::card::{
    NOTIFICATION_EXIT_ANIMATION_MS, NotificationCard, NotificationCardCallbacks,
    dropdown_card_options,
};
use crate::notifications::model::NotificationToast;

pub(super) struct NotificationRow {
    notification: NotificationToast,
    card: Option<NotificationCard>,
    revealer: Option<gtk::Revealer>,
    dismissing: bool,
}

#[derive(Debug)]
pub(super) enum NotificationRowInput {
    InvokeDefault,
    InvokeAction(String),
    Dismiss,
    DismissFinished(u32),
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

    fn init_model(
        notification: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self {
            notification,
            card: None,
            revealer: None,
            dismissing: false,
        }
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
                self.start_dismiss_animation(sender);
            }
            NotificationRowInput::DismissFinished(id) => {
                if self.dismissing && self.notification.id == id {
                    let _ = sender.output(NotificationRowOutput::Dismiss(id));
                }
            }
        }
    }
}

impl NotificationRow {
    pub(super) fn id(&self) -> u32 {
        self.notification.id
    }

    fn sync_row_widgets(&mut self, root: &gtk::Box, sender: FactorySender<Self>) {
        if let Some(card) = &self.card {
            if !self.dismissing {
                card.update(&self.notification);
            }
            return;
        }

        let default_sender = sender.clone();
        let action_sender = sender.clone();
        let dismiss_sender = sender;

        let revealer = gtk::Revealer::new();
        revealer.add_css_class("notification-list-row-revealer");
        revealer.set_transition_duration(NOTIFICATION_EXIT_ANIMATION_MS as u32);
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideUp);
        revealer.set_reveal_child(true);

        let card = NotificationCard::new(
            &self.notification,
            dropdown_card_options(),
            NotificationCardCallbacks {
                on_default: Some(std::rc::Rc::new(move |_| {
                    default_sender.input(NotificationRowInput::InvokeDefault);
                })),
                on_action: Some(std::rc::Rc::new(move |_, action_id| {
                    action_sender.input(NotificationRowInput::InvokeAction(action_id));
                })),
                on_dismiss: Some(std::rc::Rc::new(move |_| {
                    dismiss_sender.input(NotificationRowInput::Dismiss);
                })),
            },
        );

        revealer.set_child(Some(card.root()));
        root.append(&revealer);

        self.card = Some(card);
        self.revealer = Some(revealer);
    }

    pub(super) fn set_notification(&mut self, notification: NotificationToast) {
        let id_changed = self.notification.id != notification.id;
        self.notification = notification;

        if id_changed {
            self.dismissing = false;

            if let Some(card) = &self.card {
                card.set_dismissing(false);
                card.update(&self.notification);
            }

            if let Some(revealer) = &self.revealer {
                revealer.set_reveal_child(true);
            }
        } else if !self.dismissing {
            if let Some(card) = &self.card {
                card.update(&self.notification);
            }
        }
    }

    fn start_dismiss_animation(&mut self, sender: FactorySender<Self>) {
        if self.dismissing {
            return;
        }

        self.dismissing = true;

        if let Some(card) = &self.card {
            card.set_dismissing(true);
        }

        if let Some(revealer) = &self.revealer {
            revealer.set_reveal_child(false);
        }

        let id = self.notification.id;
        let input_sender = sender.input_sender().clone();

        glib::timeout_add_local_once(
            Duration::from_millis(NOTIFICATION_EXIT_ANIMATION_MS),
            move || {
                let _ = input_sender.send(NotificationRowInput::DismissFinished(id));
            },
        );
    }
}
