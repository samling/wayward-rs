use super::row::{NotificationRow, NotificationRowOutput};
use crate::bar::BarMsg;
use crate::bar::widget::{WidgetAction, WidgetEvent};
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
        #[name = "popover"]
        gtk::Popover {
            set_has_arrow: false,
            set_autohide: true,
            add_css_class: "dropdown",
            add_css_class: "notifications-dropdown",

            #[watch]
            set_position: dropdown::position_for_edge(model.edge),

            #[watch]
            set_offset: (
                dropdown::x_offset_for_placement(model.edge, model.region),
                dropdown::y_offset_for_placement(model.edge, model.region),
            ),

            #[watch]
            set_margin_start: dropdown::margin_start_for_placement(model.edge, model.region),
            #[watch]
            set_margin_end: dropdown::margin_end_for_placement(model.edge, model.region),
            #[watch]
            set_margin_top: dropdown::margin_top_for_placement(model.edge, model.region),
            #[watch]
            set_margin_bottom: dropdown::margin_bottom_for_placement(model.edge, model.region),

            #[name = "revealer"]
            gtk::Revealer {
                set_transition_duration: dropdown::TRANSITION_MS,
                set_reveal_child: false,

                #[watch]
                set_transition_type: dropdown::transition_for_edge(model.edge),

                gtk::Box {
                    add_css_class: "dropdown-content",
                    add_css_class: "notifications-dropdown-content",
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,

                    gtk::Box {
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

                    #[name = "empty_label"]
                    gtk::Label {
                        add_css_class: "notifications-empty",
                        set_halign: gtk::Align::Start,
                        set_text: "No notifications",

                        #[watch]
                        set_visible: model.notifications.is_empty(),
                    },

                    #[name = "scroller"]
                    gtk::ScrolledWindow {
                        add_css_class: "notifications-list-scroll",
                        set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                        set_min_content_width: 360,
                        set_propagate_natural_height: true,
                        set_max_content_height: 900,

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
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
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

        let model = Self {
            edge: init.edge,
            region: init.region,
            notifications: Vec::new(),
            rows,
            bar_sender: init.bar_sender,
        };

        let widgets = view_output!();

        dropdown::connect_revealer(&widgets.popover, &widgets.revealer);

        let adjustment = widgets.scroller.vadjustment();
        adjustment.set_step_increment(72.0);
        adjustment.set_page_increment(240.0);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            NotificationsDropdownInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
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
                    action: WidgetAction::InvokeNotificationDefault { id },
                }));
            }
            NotificationsDropdownInput::InvokeAction { id, action_id } => {
                let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
                    widget_id: "notifications",
                    action: WidgetAction::InvokeNotificationAction { id, action_id },
                }));
            }
            NotificationsDropdownInput::Dismiss(id) => {
                let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
                    widget_id: "notifications",
                    action: WidgetAction::DismissNotification { id },
                }));
            }
            NotificationsDropdownInput::DismissAll => {
                let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
                    widget_id: "notifications",
                    action: WidgetAction::DismissAllNotifications,
                }));
            }
        }
    }
}

impl NotificationsDropdown {
    fn sync_row_slots(&mut self, notifications: Vec<NotificationToast>) {
        let mut rows = self.rows.guard();

        let shared_len = rows.len().min(notifications.len());

        for (index, notification) in notifications.iter().take(shared_len).enumerate() {
            if let Some(row) = rows.get_mut(index) {
                row.set_notification(notification.clone());
            }
        }

        while rows.len() > notifications.len() {
            rows.pop_back();
        }

        for notification in notifications.into_iter().skip(shared_len) {
            rows.push_back(notification);
        }
    }
}
