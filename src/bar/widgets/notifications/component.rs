use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentController, Controller};

use crate::bar::BarMsg;
use crate::bar::layout::BarEdge;
use crate::bar::widget::BarRegion;
use crate::notifications::model::NotificationToast;

use super::dropdown::{NotificationsDropdown, NotificationsDropdownInit, NotificationsDropdownInput};

pub(super) struct NotificationsComponent {
    edge: BarEdge,
    region: BarRegion,
    notifications: Vec<NotificationToast>,
    dropdown: Controller<NotificationsDropdown>,
}

#[derive(Debug)]
pub(super) enum NotificationsInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetNotifications(Vec<NotificationToast>),
    SetUnavailable,
}

pub(super) struct NotificationsInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
    pub(super) bar_sender: relm4::Sender<BarMsg>,
}

#[relm4::component(pub(super))]
impl SimpleComponent for NotificationsComponent {
    type Init = NotificationsInit;
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

                    #[watch]
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
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let dropdown = NotificationsDropdown::builder()
            .launch(NotificationsDropdownInit {
                edge: init.edge,
                region: init.region,
                bar_sender: init.bar_sender,
            })
            .detach();

        let model = Self {
            edge: init.edge,
            region: init.region,
            notifications: Vec::new(),
            dropdown,
        };

        let widgets = view_output!();

        root.set_popover(Some(model.dropdown.widget()));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            NotificationsInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
                self.dropdown
                    .emit(NotificationsDropdownInput::SetPlacement { edge, region });
            }
            NotificationsInput::SetNotifications(notifications) => {
                self.dropdown
                    .emit(NotificationsDropdownInput::SetNotifications(notifications.clone()));
                self.notifications = notifications;
            }
            NotificationsInput::SetUnavailable => {
                self.dropdown.emit(NotificationsDropdownInput::SetUnavailable);
                self.notifications.clear();
            }
        }
    }
}