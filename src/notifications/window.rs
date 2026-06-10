use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk::{self, glib};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use super::card::{
    NOTIFICATION_EXIT_ANIMATION_MS, NotificationCard, NotificationCardCallbacks, toast_card_options,
};
use super::model::NotificationToast;
use crate::shell::ShellMsg;

const TOP_MARGIN: i32 = 8;
const RIGHT_MARGIN: i32 = 12;
const STACK_SPACING: i32 = 8;

pub(crate) struct NotificationWindow {
    window: gtk::Window,
    stack: gtk::Box,
    sender: relm4::Sender<ShellMsg>,
    rows: Rc<RefCell<Vec<NotificationToastRow>>>,
}

struct NotificationToastRow {
    id: u32,
    revealer: gtk::Revealer,
    card: NotificationCard,
    dismissing: Cell<bool>,
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
            rows: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(crate) fn set_toasts(&self, toasts: &[NotificationToast]) {
        let mut missing_ids = Vec::new();

        {
            let rows = self.rows.borrow();

            for row in rows.iter() {
                if let Some(toast) = toasts.iter().find(|toast| toast.id == row.id) {
                    row.dismissing.set(false);
                    row.card.set_dismissing(false);
                    row.card.update(toast);
                    row.revealer.set_reveal_child(true);
                } else {
                    missing_ids.push(row.id);
                }
            }
        }

        for toast in toasts {
            let exists = self.rows.borrow().iter().any(|row| row.id == toast.id);

            if !exists {
                let row = self.toast_row(toast);
                self.stack.append(&row.revealer);
                self.rows.borrow_mut().push(row);
            }
        }

        if missing_ids.is_empty() {
            self.reorder_toasts(toasts);
        }

        for id in missing_ids {
            self.start_toast_exit(id);
        }

        self.window
            .set_visible(!toasts.is_empty() || !self.rows.borrow().is_empty());
    }

    fn toast_row(&self, toast: &NotificationToast) -> NotificationToastRow {
        let default_sender = self.sender.clone();
        let action_sender = self.sender.clone();
        let dismiss_sender = self.sender.clone();

        let revealer = gtk::Revealer::new();
        revealer.add_css_class("notification-toast-revealer");
        revealer.set_transition_duration(NOTIFICATION_EXIT_ANIMATION_MS as u32);
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideUp);
        revealer.set_reveal_child(true);

        let card = NotificationCard::new(
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
        );

        revealer.set_child(Some(card.root()));

        NotificationToastRow {
            id: toast.id,
            revealer,
            card,
            dismissing: Cell::new(false),
        }
    }

    fn reorder_toasts(&self, toasts: &[NotificationToast]) {
        let mut previous: Option<gtk::Widget> = None;

        for toast in toasts {
            let revealer = self
                .rows
                .borrow()
                .iter()
                .find(|row| row.id == toast.id && !row.dismissing.get())
                .map(|row| row.revealer.clone().upcast::<gtk::Widget>());

            if let Some(revealer) = revealer {
                self.stack.reorder_child_after(&revealer, previous.as_ref());
                previous = Some(revealer);
            }
        }
    }

    fn start_toast_exit(&self, id: u32) {
        let Some(revealer) = self.mark_toast_exiting(id) else {
            return;
        };

        let rows = self.rows.clone();
        let stack = self.stack.clone();
        let window = self.window.clone();

        glib::timeout_add_local_once(
            Duration::from_millis(NOTIFICATION_EXIT_ANIMATION_MS),
            move || {
                let removed = {
                    let mut rows = rows.borrow_mut();
                    rows.iter()
                        .position(|row| row.id == id && row.dismissing.get())
                        .map(|index| rows.remove(index))
                };

                if removed.is_some() {
                    stack.remove(&revealer);
                }

                if rows.borrow().is_empty() {
                    window.set_visible(false);
                }
            },
        );
    }

    fn mark_toast_exiting(&self, id: u32) -> Option<gtk::Revealer> {
        let rows = self.rows.borrow();
        let row = rows.iter().find(|row| row.id == id)?;

        if row.dismissing.replace(true) {
            return None;
        }

        row.card.set_dismissing(true);
        row.revealer.set_reveal_child(false);

        Some(row.revealer.clone())
    }
}
