use crate::bar::{dropdown, layout::BarEdge, widget::BarRegion};
use crate::notifications::model::NotificationToast;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use std::cell::RefCell;

pub(super) struct NotificationsDropdown {
    edge: BarEdge,
    region: BarRegion,
    notifications: Vec<NotificationToast>,
    rendered_notification_ids: RefCell<Vec<u32>>,
}

pub(super) struct NotificationsDropdownInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
}

#[derive(Debug)]
pub(super) enum NotificationsDropdownInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetNotifications(Vec<NotificationToast>),
    SetUnavailable,
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

                            #[name = "list"]
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 6,
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            edge: init.edge,
            region: init.region,
            notifications: Vec::new(),
            rendered_notification_ids: RefCell::new(Vec::new()),
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
                self.notifications = notifications;
            }
            NotificationsDropdownInput::SetUnavailable => {
                self.notifications.clear();
            }
        }
    }

    fn post_view() {
        let notification_ids = self
            .notifications
            .iter()
            .map(|notification| notification.id)
            .collect::<Vec<_>>();

        if *self.rendered_notification_ids.borrow() == notification_ids {
            return;
        }

        while let Some(child) = widgets.list.first_child() {
            widgets.list.remove(&child);
        }

        for notification in &self.notifications {
            widgets.list.append(&notification_row(notification));
        }

        self.rendered_notification_ids.replace(notification_ids);
    }
}

fn notification_row(notification: &NotificationToast) -> gtk::Widget {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("notification-list-row");
    row.set_width_request(320);

    let icon = gtk::Image::from_icon_name(&notification.app_icon);
    icon.add_css_class("notification-list-icon");
    icon.set_valign(gtk::Align::Start);
    row.append(&icon);

    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.add_css_class("notification-list-text");
    text.set_hexpand(true);

    let app_name = gtk::Label::new(Some(&notification.app_name));
    app_name.add_css_class("notification-list-app-name");
    app_name.set_halign(gtk::Align::Start);
    app_name.set_xalign(0.0);
    app_name.set_max_width_chars(34);
    app_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    text.append(&app_name);

    let summary = gtk::Label::new(Some(&notification.summary));
    summary.add_css_class("notification-list-summary");
    summary.set_halign(gtk::Align::Start);
    summary.set_hexpand(true);
    summary.set_xalign(0.0);
    summary.set_wrap(true);
    summary.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    summary.set_max_width_chars(34);
    text.append(&summary);

    if let Some(body) = &notification.body {
        let body = gtk::Label::new(Some(body));
        body.add_css_class("notification-list-body");
        body.set_halign(gtk::Align::Start);
        body.set_hexpand(true);
        body.set_xalign(0.0);
        body.set_wrap(true);
        body.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        body.set_max_width_chars(34);
        body.set_lines(3);
        body.set_ellipsize(gtk::pango::EllipsizeMode::End);
        text.append(&body);
    }

    row.append(&text);

    row.upcast()
}