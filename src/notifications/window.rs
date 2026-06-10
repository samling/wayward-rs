use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;
use std::cell::RefCell;
use std::rc::Rc;

use super::card::{NotificationCardCallbacks, notification_card, toast_card_options};
use super::model::NotificationToast;
use crate::shell::ShellMsg;

const TOP_MARGIN: i32 = 8;
const RIGHT_MARGIN: i32 = 12;
const STACK_SPACING: i32 = 8;

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
        let default_sender = self.sender.clone();
        let action_sender = self.sender.clone();
        let dismiss_sender = self.sender.clone();

        notification_card(
            toast,
            toast_card_options(),
            NotificationCardCallbacks {
                on_default: Some(Rc::new(move |id| {
                    let _ = default_sender.send(ShellMsg::InvokeNotificationDefaultAction(id));
                })),
                on_action: Some(Rc::new(move |id, action_id| {
                    let _ =
                        action_sender.send(ShellMsg::InvokeNotificationAction { id, action_id });
                })),
                on_dismiss: Some(Rc::new(move |id| {
                    let _ = dismiss_sender.send(ShellMsg::DismissNotificationPopup(id));
                })),
            },
        )
        .upcast()
    }
}
