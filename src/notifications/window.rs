use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;
use std::cell::RefCell;

use super::model::NotificationToast;
use crate::shell::ShellMsg;

const TOP_MARGIN: i32 = 36;
const RIGHT_MARGIN: i32 = 12;
const STACK_SPACING: i32 = 8;
const TEXT_WIDTH_CHARS: i32 = 42;
const SUMMARY_MAX_LINES: i32 = 2;
const BODY_MAX_LINES: i32 = 4;

fn configure_wrapping_label(label: &gtk::Label, max_lines: i32) {
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label.set_width_chars(TEXT_WIDTH_CHARS);
    label.set_max_width_chars(TEXT_WIDTH_CHARS);
    label.set_lines(max_lines);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.set_xalign(0.0);
}

pub(crate) struct NotificationWindow {
    window: gtk::Window,
    stack: gtk::Box,
    sender: relm4::Sender<ShellMsg>,
    toasts: RefCell<Vec<NotificationToast>>,
}

impl NotificationWindow {
    pub(crate) fn new(monitor: &gtk::gdk::Monitor, sender: relm4::Sender<ShellMsg>) -> Self {
        let window = gtk::Window::new();
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_monitor(Some(monitor));
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, TOP_MARGIN);
        window.set_margin(Edge::Right, RIGHT_MARGIN);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_namespace(Some("wayward-notifications"));
        window.set_visible(false);
        window.add_css_class("notification-window");

        let stack = gtk::Box::new(gtk::Orientation::Vertical, STACK_SPACING);
        stack.add_css_class("notification-stack");
        stack.set_halign(gtk::Align::End);
        stack.set_valign(gtk::Align::Start);

        window.set_child(Some(&stack));

        Self {
            window,
            stack,
            sender,
            toasts: RefCell::new(Vec::new()),
        }
    }

    fn actions(&self, toast_id: u32, actions: &[super::model::NotificationAction]) -> gtk::Widget {
        let row = gtk::FlowBox::new();
        row.add_css_class("notification-actions");
        row.set_selection_mode(gtk::SelectionMode::None);
        row.set_max_children_per_line(3);

        for action in actions {
            let button = gtk::Button::with_label(&action.label);
            button.add_css_class("notification-action");
            button.add_css_class("flat");

            let sender = self.sender.clone();
            let action_id = action.id.clone();

            button.connect_clicked(move |_| {
                let _ = sender.send(ShellMsg::InvokeNotificationAction {
                    id: toast_id,
                    action_id: action_id.clone(),
                });
            });

            row.insert(&button, -1);
        }

        row.upcast()
    }

    pub(crate) fn set_toasts(&self, toasts: &[NotificationToast]) {
        let unchanged = { self.toasts.borrow().as_slice() == toasts };

        if unchanged {
            return;
        }

        self.toasts.replace(toasts.to_vec());

        while let Some(child) = self.stack.first_child() {
            self.stack.remove(&child);
        }

        for toast in toasts {
            self.stack.append(&self.toast_widget(toast));
        }

        self.window.set_visible(!toasts.is_empty());
    }

    fn toast_widget(&self, toast: &NotificationToast) -> gtk::Widget {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.add_css_class("notification-toast");
        root.add_css_class(toast.urgency_class());

        root.append(&self.header(toast));
        root.append(&self.body(toast));

        let actions = toast.visible_actions();
        if !actions.is_empty() {
            root.append(&self.actions(toast.id, &actions));
        }

        root.upcast()
    }

    fn header(&self, toast: &NotificationToast) -> gtk::Widget {
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.add_css_class("notification-header");

        let icon = gtk::Image::new();
        icon.add_css_class("notification-app-icon");
        crate::notifications::icon::set_notification_icon(&icon, toast);
        header.append(&icon);

        let app_name = gtk::Label::new(Some(&toast.app_name));
        app_name.add_css_class("notification-app-name");
        app_name.set_hexpand(true);
        app_name.set_halign(gtk::Align::Start);
        app_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        header.append(&app_name);

        let close = gtk::Button::new();
        close.add_css_class("notification-close");
        close.add_css_class("flat");
        close.set_child(Some(&gtk::Image::from_icon_name("window-close-symbolic")));

        let sender = self.sender.clone();
        let id = toast.id;
        close.connect_clicked(move |_| {
            let _ = sender.send(ShellMsg::DismissNotificationPopup(id));
        });

        header.append(&close);
        header.upcast()
    }

    fn body(&self, toast: &NotificationToast) -> gtk::Widget {
        let body = gtk::Box::new(gtk::Orientation::Vertical, 4);
        body.add_css_class("notification-content");

        let summary = gtk::Label::new(Some(&toast.summary));
        summary.add_css_class("notification-summary");
        summary.set_halign(gtk::Align::Start);
        configure_wrapping_label(&summary, SUMMARY_MAX_LINES);
        body.append(&summary);

        if let Some(text) = &toast.body {
            let label = gtk::Label::new(Some(text));
            label.add_css_class("notification-body");
            label.set_halign(gtk::Align::Start);
            configure_wrapping_label(&label, BODY_MAX_LINES);
            body.append(&label);
        }

        if !toast.has_default_action() {
            return body.upcast();
        }

        let button = gtk::Button::new();
        button.add_css_class("notification-content-button");
        button.add_css_class("flat");
        button.set_child(Some(&body));

        let sender = self.sender.clone();
        let id = toast.id;
        button.connect_clicked(move |_| {
            let _ = sender.send(ShellMsg::InvokeNotificationDefaultAction(id));
        });

        button.upcast()
    }
}
