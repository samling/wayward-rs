use super::row::{NotificationRow, NotificationRowOutput};
use crate::bar::BarMsg;
use crate::bar::widget::{NotificationAction, WidgetAction, WidgetEvent};
use crate::bar::{dropdown, layout::BarEdge, widget::BarRegion};
use crate::notifications::model::NotificationToast;
use relm4::factory::FactoryVecDeque;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

pub(super) struct NotificationsDropdown {
    edge: BarEdge,
    region: BarRegion,
    notifications: Vec<NotificationToast>,
    rows: FactoryVecDeque<NotificationRow>,
    bar_sender: relm4::Sender<BarMsg>,
    shell: Option<dropdown::DropdownPopover>,
}

pub(super) struct NotificationsDropdownInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
    pub(super) bar_sender: relm4::Sender<BarMsg>,
}

#[derive(Debug)]
pub(super) enum NotificationsDropdownInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetNotifications(Vec<NotificationToast>),
    SetUnavailable,
    InvokeDefault(u32),
    InvokeAction { id: u32, action_id: String },
    Dismiss(u32),
    DismissAll,
}

#[relm4::component(pub(super))]
impl SimpleComponent for NotificationsDropdown {
    type Init = NotificationsDropdownInit;
    type Input = NotificationsDropdownInput;
    type Output = ();

    view! {
        #[root]
        #[template]
        #[name = "shell"]
        dropdown::DropdownPopover(dropdown::DropdownPopoverInit {
            root_css_class: "notifications-dropdown",
            content_css_class: "notifications-dropdown-content",
            content_spacing: 8,
        }) {
            #[template_child]
            content {

                    gtk::Box {
                        add_css_class: "dropdown-header",
                        add_css_class: "notifications-dropdown-header",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                        set_hexpand: true,

                        gtk::Label {
                            add_css_class: "dropdown-title",
                            add_css_class: "notifications-dropdown-title",
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                            set_text: "Notifications",
                        },

                        gtk::Button {
                            add_css_class: "notification-clear-all",
                            add_css_class: "flat",
                            set_label: "Clear all",

                            #[watch]
                            set_visible: !model.notifications.is_empty(),

                            connect_clicked[sender] => move |_| {
                                sender.input(NotificationsDropdownInput::DismissAll);
                            }
                        }
                    },

                    gtk::Box {
                        add_css_class: "dropdown-empty",
                        add_css_class: "notifications-empty",
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 0,
                        set_halign: gtk::Align::Fill,

                        #[watch]
                        set_visible: model.notifications.is_empty(),

                        gtk::Box {
                            set_vexpand: true,
                        },

                        gtk::Box {
                            add_css_class: "dropdown-empty-content",
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 6,
                            set_halign: gtk::Align::Center,

                            gtk::Image {
                                add_css_class: "dropdown-empty-icon",
                                set_halign: gtk::Align::Center,
                                set_icon_name: Some("preferences-system-notifications-symbolic"),
                            },

                            gtk::Label {
                                add_css_class: "dropdown-empty-title",
                                set_halign: gtk::Align::Center,
                                set_justify: gtk::Justification::Center,
                                set_text: "No notifications",
                            },

                            gtk::Label {
                                add_css_class: "dropdown-empty-subtitle",
                                set_halign: gtk::Align::Center,
                                set_justify: gtk::Justification::Center,
                                set_text: "You're all caught up",
                            },
                        },

                        gtk::Box {
                            set_vexpand: true,
                        },
                    },

                    #[name = "scroller"]
                    gtk::ScrolledWindow {
                        add_css_class: "notifications-list-scroll",
                        set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                        set_kinetic_scrolling: true,
                        set_min_content_width: 360,
                        set_propagate_natural_height: true,
                        set_max_content_height: 900,

                        #[watch]
                        set_visible: !model.notifications.is_empty(),

                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            add_css_class: "notifications-list",
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 6,

                            #[local_ref]
                            list -> gtk::ListBox {
                                add_css_class: "notifications-list-items",
                                set_selection_mode: gtk::SelectionMode::None,
                            }
                        },
                    },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list = gtk::ListBox::default();
        let rows = FactoryVecDeque::builder().launch(list.clone()).forward(
            sender.input_sender(),
            |output| match output {
                NotificationRowOutput::InvokeDefault(id) => {
                    NotificationsDropdownInput::InvokeDefault(id)
                }
                NotificationRowOutput::InvokeAction { id, action_id } => {
                    NotificationsDropdownInput::InvokeAction { id, action_id }
                }
                NotificationRowOutput::Dismiss(id) => NotificationsDropdownInput::Dismiss(id),
            },
        );

        let mut model = Self {
            edge: init.edge,
            region: init.region,
            notifications: Vec::new(),
            rows,
            bar_sender: init.bar_sender,
            shell: None,
        };

        let widgets = view_output!();

        root.set_placement(init.edge, init.region);
        root.connect_revealer();
        model.shell = Some(root);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            NotificationsDropdownInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
                if let Some(shell) = &self.shell {
                    shell.set_placement(edge, region);
                }
            }
            NotificationsDropdownInput::SetNotifications(notifications) => {
                self.notifications = notifications.clone();
                self.sync_row_slots(notifications);
            }
            NotificationsDropdownInput::SetUnavailable => {
                self.notifications.clear();
                self.rows.guard().clear();
            }
            NotificationsDropdownInput::InvokeDefault(id) => {
                let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
                    widget_id: "notifications",
                    action: WidgetAction::Notifications(NotificationAction::InvokeDefault { id }),
                }));
            }
            NotificationsDropdownInput::InvokeAction { id, action_id } => {
                let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
                    widget_id: "notifications",
                    action: WidgetAction::Notifications(NotificationAction::InvokeAction {
                        id,
                        action_id,
                    }),
                }));
            }
            NotificationsDropdownInput::Dismiss(id) => {
                let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
                    widget_id: "notifications",
                    action: WidgetAction::Notifications(NotificationAction::Dismiss { id }),
                }));
            }
            NotificationsDropdownInput::DismissAll => {
                let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
                    widget_id: "notifications",
                    action: WidgetAction::Notifications(NotificationAction::DismissAll),
                }));
            }
        }
    }
}

impl NotificationsDropdown {
    fn sync_row_slots(&mut self, notifications: Vec<NotificationToast>) {
        let mut rows = self.rows.guard();
        let target_ids = notifications
            .iter()
            .map(|notification| notification.id)
            .collect::<Vec<_>>();

        for index in (0..rows.len()).rev() {
            if !target_ids.contains(&rows[index].id()) {
                rows.remove(index);
            }
        }

        for (target_index, notification) in notifications.iter().enumerate() {
            if target_index < rows.len() && rows[target_index].id() == notification.id {
                if let Some(row) = rows.get_mut(target_index) {
                    row.set_notification(notification.clone());
                }
                continue;
            }

            let existing_index = rows.iter().position(|row| row.id() == notification.id);

            if let Some(existing_index) = existing_index {
                rows.move_to(existing_index, target_index);
                if let Some(row) = rows.get_mut(target_index) {
                    row.set_notification(notification.clone());
                }
            } else {
                rows.insert(target_index, notification.clone());
            }
        }

        while rows.len() > notifications.len() {
            rows.pop_back();
        }
    }
}
