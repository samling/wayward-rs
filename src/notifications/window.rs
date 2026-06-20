use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk::{self, glib};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::card::{
    NOTIFICATION_EXIT_ANIMATION_MS, NotificationCard, NotificationCardCallbacks, toast_card_options,
};
use super::model::NotificationToast;
use crate::shell::ShellMsg;

const TOP_MARGIN: i32 = 8;
const RIGHT_MARGIN: i32 = 12;
const STACK_SPACING: i32 = 8;
const NOTIFICATION_ENTER_ANIMATION_MS: u64 = 80;

pub(crate) struct NotificationWindow {
    monitor: gtk::gdk::Monitor,
    sender: relm4::Sender<ShellMsg>,
    rows: Rc<RefCell<Vec<NotificationToastRow>>>,
}

struct NotificationToastRow {
    id: u32,
    window: gtk::Window,
    width: i32,
    height: i32,
    card: NotificationCard,
    dismissing: Cell<bool>,
}

impl NotificationWindow {
    pub(crate) fn new(monitor: &gtk::gdk::Monitor, sender: relm4::Sender<ShellMsg>) -> Self {
        Self {
            monitor: monitor.clone(),
            sender,
            rows: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(crate) fn set_toasts(&self, toasts: &[NotificationToast]) {
        let mut missing_ids = Vec::new();
        let mut entering_rows = Vec::new();

        {
            let rows = self.rows.borrow();

            for row in rows.iter() {
                if let Some(toast) = toasts.iter().find(|toast| toast.id == row.id) {
                    row.dismissing.set(false);
                    row.card.set_dismissing(false);
                    row.card.update(toast);
                } else {
                    missing_ids.push(row.id);
                }
            }
        }

        let mut top_margin = TOP_MARGIN;

        for toast in toasts {
            let existing_index = self
                .rows
                .borrow()
                .iter()
                .position(|row| row.id == toast.id && !row.dismissing.get());

            if let Some(index) = existing_index {
                let rows = self.rows.borrow();
                let row = &rows[index];
                row.window.set_margin(Edge::Top, top_margin);
                row.window.set_margin(Edge::Right, RIGHT_MARGIN);
                top_margin += row.height + STACK_SPACING;
            } else {
                let row = self.toast_row(toast);
                row.window.set_margin(Edge::Top, top_margin);
                row.window.set_margin(Edge::Right, -row.width);
                row.window.set_visible(true);

                entering_rows.push((row.window.clone(), row.width));
                top_margin += row.height + STACK_SPACING;
                self.rows.borrow_mut().push(row);
            }
        }

        for (window, width) in entering_rows {
            start_toast_enter(&window, width);
        }

        for id in missing_ids {
            self.start_toast_exit(id);
        }
    }

    fn toast_row(&self, toast: &NotificationToast) -> NotificationToastRow {
        let default_sender = self.sender.clone();
        let action_sender = self.sender.clone();
        let dismiss_sender = self.sender.clone();

        let window = gtk::Window::new();
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_monitor(Some(&self.monitor));
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_namespace(Some("wayward-notifications"));
        window.set_visible(false);
        window.add_css_class("notification-window");

        let options = toast_card_options();
        let card = NotificationCard::new(
            toast,
            options,
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

        let width = options.width_request.unwrap_or(320);
        let (_, height, _, _) = card.root().measure(gtk::Orientation::Vertical, width);
        window.set_child(Some(card.root()));

        NotificationToastRow {
            id: toast.id,
            window,
            width,
            height,
            card,
            dismissing: Cell::new(false),
        }
    }

    fn start_toast_exit(&self, id: u32) {
        let Some(window) = self.mark_toast_exiting(id) else {
            return;
        };

        let rows = self.rows.clone();

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
                    window.set_visible(false);
                }
            },
        );
    }

    fn mark_toast_exiting(&self, id: u32) -> Option<gtk::Window> {
        let rows = self.rows.borrow();
        let row = rows.iter().find(|row| row.id == id)?;

        if row.dismissing.replace(true) {
            return None;
        }

        row.card.set_dismissing(true);

        Some(row.window.clone())
    }
}

fn start_toast_enter(window: &gtk::Window, width: i32) {
    let window = window.clone();

    glib::idle_add_local_once(move || {
        let started_at = Instant::now();

        glib::timeout_add_local(Duration::from_millis(16), move || {
            let elapsed_ms = started_at.elapsed().as_millis() as f64;
            let progress = (elapsed_ms / NOTIFICATION_ENTER_ANIMATION_MS as f64).clamp(0.0, 1.0);

            window.set_margin(Edge::Right, enter_animation_right_margin(progress, width));

            if progress >= 1.0 {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    });
}

fn enter_animation_right_margin(progress: f64, width: i32) -> i32 {
    let progress = progress.clamp(0.0, 1.0);
    let start = -width;
    let end = RIGHT_MARGIN;

    (start as f64 + (end - start) as f64 * progress).round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_animation_moves_from_offscreen_right_to_resting_position() {
        assert_eq!(enter_animation_right_margin(0.0, 320), -320);
        assert_eq!(enter_animation_right_margin(1.0, 320), RIGHT_MARGIN);
    }
}
