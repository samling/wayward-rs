use relm4::gtk;
use relm4::gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::model::{NotificationAction, NotificationToast};

const DROPDOWN_TEXT_WIDTH_CHARS: i32 = 34;
const DROPDOWN_BODY_PREVIEW_LINES: usize = 4;
const NOTIFICATION_CARD_WIDTH: i32 = 392;
pub(crate) const NOTIFICATION_EXIT_ANIMATION_MS: u64 = 220;

#[derive(Clone, Default)]
pub(crate) struct NotificationCardCallbacks {
    pub(crate) on_default: Option<Rc<dyn Fn(u32)>>,
    pub(crate) on_action: Option<Rc<dyn Fn(u32, String)>>,
    pub(crate) on_dismiss: Option<Rc<dyn Fn(u32)>>,
}

#[derive(Clone, Copy)]
pub(crate) struct NotificationCardOptions {
    pub(crate) class_name: &'static str,
    pub(crate) width_request: Option<i32>,
    pub(crate) text_width_chars: i32,
    pub(crate) summary_lines: i32,
    pub(crate) body_lines: i32,
    pub(crate) body_preview_lines: Option<usize>,
}

pub(crate) fn dropdown_card_options() -> NotificationCardOptions {
    NotificationCardOptions {
        class_name: "notification-list-row",
        width_request: Some(NOTIFICATION_CARD_WIDTH),
        text_width_chars: DROPDOWN_TEXT_WIDTH_CHARS,
        summary_lines: 2,
        body_lines: 3,
        body_preview_lines: Some(DROPDOWN_BODY_PREVIEW_LINES),
    }
}

pub(crate) fn toast_card_options() -> NotificationCardOptions {
    NotificationCardOptions {
        class_name: "notification-toast",
        width_request: Some(NOTIFICATION_CARD_WIDTH),
        text_width_chars: DROPDOWN_TEXT_WIDTH_CHARS,
        summary_lines: 2,
        body_lines: 4,
        body_preview_lines: None,
    }
}

pub(crate) struct NotificationCard {
    root: gtk::Box,
    icon: gtk::Image,
    app_name: gtk::Label,
    summary: gtk::Label,
    body: gtk::Label,
    message: gtk::Box,
    actions: gtk::Box,
    current_id: Rc<Cell<u32>>,
    has_default_action: Rc<Cell<bool>>,
    current_actions: RefCell<Vec<NotificationAction>>,
    callbacks: NotificationCardCallbacks,
    options: NotificationCardOptions,
    urgency_class: Cell<&'static str>,
}

impl NotificationCard {
    pub(crate) fn new(
        notification: &NotificationToast,
        options: NotificationCardOptions,
        callbacks: NotificationCardCallbacks,
    ) -> Self {
        let current_id = Rc::new(Cell::new(notification.id));
        let has_default_action = Rc::new(Cell::new(notification.has_default_action()));

        let root = root_widget(notification, options);
        let icon = icon_widget(notification);
        root.append(&icon);

        let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
        text.add_css_class("notification-card-text");
        text.set_hexpand(true);

        let message = message_widget(notification, options, &callbacks);
        text.append(&message);

        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        actions.add_css_class("notification-card-actions");
        text.append(&actions);

        root.append(&text);

        if let Some(on_default) = callbacks.on_default.clone() {
            connect_default_action(
                &message,
                current_id.clone(),
                has_default_action.clone(),
                on_default,
            );
        }

        if let Some(on_dismiss) = callbacks.on_dismiss.clone() {
            root.append(&dismiss_button(current_id.clone(), on_dismiss));
        }

        let card = Self {
            root,
            icon,
            app_name: app_name_label(&message),
            summary: summary_label(&message),
            body: body_label(&message),
            message,
            actions,
            current_id,
            has_default_action,
            current_actions: RefCell::new(Vec::new()),
            callbacks,
            options,
            urgency_class: Cell::new(notification.urgency_class()),
        };

        card.update(notification);
        card
    }

    pub(crate) fn root(&self) -> &gtk::Box {
        &self.root
    }

    pub(crate) fn update(&self, notification: &NotificationToast) {
        self.current_id.set(notification.id);
        self.has_default_action
            .set(notification.has_default_action());
        self.root.remove_css_class("dismissing");

        let previous_urgency = self.urgency_class.replace(notification.urgency_class());
        if previous_urgency != notification.urgency_class() {
            self.root.remove_css_class(previous_urgency);
            self.root.add_css_class(notification.urgency_class());
        }

        crate::notifications::icon::set_notification_icon(&self.icon, notification);
        self.app_name.set_label(&notification.app_name);
        self.summary.set_label(&notification.summary);
        update_body_label(&self.body, notification, self.options);
        update_message_action_state(&self.message, notification.has_default_action());
        self.update_actions(notification);
    }

    pub(crate) fn set_dismissing(&self, dismissing: bool) {
        if dismissing {
            self.root.add_css_class("dismissing");
        } else {
            self.root.remove_css_class("dismissing");
        }
    }

    pub(crate) fn set_entering(&self, entering: bool) {
        if entering {
            self.root.add_css_class("entering");
        } else {
            self.root.remove_css_class("entering");
        }
    }

    fn update_actions(&self, notification: &NotificationToast) {
        let actions = notification.visible_actions();
        if *self.current_actions.borrow() == actions {
            self.actions.set_visible(!actions.is_empty());
            return;
        }

        self.current_actions.replace(actions.clone());

        while let Some(child) = self.actions.first_child() {
            self.actions.remove(&child);
        }

        let Some(on_action) = self.callbacks.on_action.clone() else {
            self.actions.set_visible(false);
            return;
        };

        self.actions.set_visible(!actions.is_empty());

        for action in actions {
            let button = gtk::Button::with_label(&action.label);
            button.add_css_class("notification-card-action");
            button.add_css_class("flat");

            let action_id = action.id;
            let current_id = self.current_id.clone();
            let on_action = on_action.clone();
            button.connect_clicked(move |_| {
                on_action(current_id.get(), action_id.clone());
            });

            self.actions.append(&button);
        }
    }
}

fn root_widget(notification: &NotificationToast, options: NotificationCardOptions) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    root.add_css_class("notification-card");
    root.add_css_class(options.class_name);
    root.add_css_class(notification.urgency_class());

    if let Some(width) = options.width_request {
        root.set_width_request(width);
    }

    root
}

fn icon_widget(notification: &NotificationToast) -> gtk::Image {
    let icon = gtk::Image::new();
    icon.add_css_class("notification-card-icon");
    icon.set_valign(gtk::Align::Start);
    crate::notifications::icon::set_notification_icon(&icon, notification);
    icon
}

fn message_widget(
    notification: &NotificationToast,
    options: NotificationCardOptions,
    _callbacks: &NotificationCardCallbacks,
) -> gtk::Box {
    let message = gtk::Box::new(gtk::Orientation::Vertical, 4);
    message.add_css_class("notification-card-message");

    let app_name = gtk::Label::new(Some(&notification.app_name));
    app_name.add_css_class("notification-card-app-name");
    app_name.set_widget_name("notification-card-app-name");
    app_name.set_halign(gtk::Align::Start);
    app_name.set_xalign(0.0);
    app_name.set_max_width_chars(options.text_width_chars);
    app_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    message.append(&app_name);

    let summary = gtk::Label::new(Some(&notification.summary));
    summary.add_css_class("notification-card-summary");
    summary.set_widget_name("notification-card-summary");
    configure_wrapping_label(&summary, options.text_width_chars, options.summary_lines);
    message.append(&summary);

    let body = gtk::Label::new(None);
    body.add_css_class("notification-card-body");
    body.set_widget_name("notification-card-body");
    configure_wrapping_label(&body, options.text_width_chars, options.body_lines);
    update_body_label(&body, notification, options);
    message.append(&body);

    update_message_action_state(&message, notification.has_default_action());
    message
}

fn connect_default_action(
    message: &gtk::Box,
    current_id: Rc<Cell<u32>>,
    has_default_action: Rc<Cell<bool>>,
    on_default: Rc<dyn Fn(u32)>,
) {
    let click = gtk::GestureClick::new();
    click.connect_released(move |gesture, _, _, _| {
        if !has_default_action.get() {
            return;
        }

        gesture.set_state(gtk::EventSequenceState::Claimed);
        on_default(current_id.get());
    });

    message.add_controller(click);
}

fn dismiss_button(current_id: Rc<Cell<u32>>, on_dismiss: Rc<dyn Fn(u32)>) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("notification-card-dismiss");
    button.add_css_class("flat");
    button.set_valign(gtk::Align::Start);
    button.set_icon_name("window-close-symbolic");

    button.connect_clicked(move |_| {
        on_dismiss(current_id.get());
    });

    button
}

fn update_body_label(
    label: &gtk::Label,
    notification: &NotificationToast,
    options: NotificationCardOptions,
) {
    if let Some(body) = body_text(notification, options) {
        label.set_label(&body);
        label.set_visible(true);
    } else {
        label.set_label("");
        label.set_visible(false);
    }
}

fn update_message_action_state(message: &gtk::Box, has_default_action: bool) {
    if has_default_action {
        message.add_css_class("notification-card-message-button");
        message.set_cursor_from_name(Some("pointer"));
    } else {
        message.remove_css_class("notification-card-message-button");
        message.set_cursor_from_name(None);
    }
}

fn app_name_label(message: &gtk::Box) -> gtk::Label {
    named_child(message, "notification-card-app-name")
}

fn summary_label(message: &gtk::Box) -> gtk::Label {
    named_child(message, "notification-card-summary")
}

fn body_label(message: &gtk::Box) -> gtk::Label {
    named_child(message, "notification-card-body")
}

fn named_child(parent: &gtk::Box, name: &str) -> gtk::Label {
    let mut child = parent.first_child();

    while let Some(widget) = child {
        if widget.widget_name() == name {
            return widget
                .downcast::<gtk::Label>()
                .expect("named child is a label");
        }

        child = widget.next_sibling();
    }

    panic!("missing notification card child: {name}");
}

fn configure_wrapping_label(label: &gtk::Label, width_chars: i32, max_lines: i32) {
    label.set_hexpand(true);
    label.set_halign(gtk::Align::Fill);
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label.set_natural_wrap_mode(gtk::NaturalWrapMode::Word);
    label.set_width_chars(width_chars);
    label.set_max_width_chars(width_chars);
    label.set_lines(max_lines);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.set_xalign(0.0);
}

fn body_text(notification: &NotificationToast, options: NotificationCardOptions) -> Option<String> {
    let body = notification.body.as_deref()?.trim();

    if body.is_empty() {
        return None;
    }

    match options.body_preview_lines {
        Some(max_lines) => Some(compact_preview_text(body, max_lines)),
        None => Some(body.to_string()),
    }
}

fn compact_preview_text(value: &str, max_lines: usize) -> String {
    let mut lines = value.lines().take(max_lines).collect::<Vec<_>>();

    if value.lines().count() > max_lines {
        if let Some(last_line) = lines.last_mut() {
            *last_line = "...";
        }
    }

    lines.join("\n")
}
