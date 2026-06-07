use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

use crate::notifications::model::NotificationToast;

pub(super) struct NotificationsComponent {
    notifications: Vec<NotificationToast>,
}

#[derive(Debug)]
pub(super) enum NotificationsInput {
    SetNotifications(Vec<NotificationToast>),
    SetUnavailable,
}

#[relm4::component(pub(super))]
impl SimpleComponent for NotificationsComponent {
    type Init = ();
    type Input = NotificationsInput;
    type Output = ();

    view! {
        gtk::MenuButton {
            set_always_show_arrow: false,
            add_css_class: "bar-item",
            add_css_class: "notifications",
            add_css_class: "flat",

            #[wrap(Some)]
            set_child = &gtk::Box {
                add_css_class: "bar-item-content",
                add_css_class: "notifications-content",
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 4,

                gtk::Image {
                    add_css_class: "notifications-icon",
                    set_icon_name: Some("preferences-system-notifications-symbolic"),
                },

                gtk::Label {
                    add_css_class: "notifications-count",

                    #[watch]
                    set_visible: !model.notifications.is_empty(),

                    #[watch]
                    set_text: &model.notifications.len().to_string(),
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            notifications: Vec::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            NotificationsInput::SetNotifications(notifications) => {
                self.notifications = notifications;
            }
            NotificationsInput::SetUnavailable => {
                self.notifications.clear();
            }
        }
    }
}