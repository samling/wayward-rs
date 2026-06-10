use relm4::gtk;
use relm4::gtk::prelude::*;
use std::rc::Rc;

use super::model::NotificationToast;

const DROPDOWN_TEXT_WIDTH_CHARS: i32 = 34;
const DROPDOWN_BODY_PREVIEW_LINES: usize = 4;

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
        width_request: Some(320),
        text_width_chars: DROPDOWN_TEXT_WIDTH_CHARS,
        summary_lines: 2,
        body_lines: 3,
        body_preview_lines: Some(DROPDOWN_BODY_PREVIEW_LINES),
    }
}

pub(crate) fn toast_card_options() -> NotificationCardOptions {
    NotificationCardOptions {
        class_name: "notification-toast",
        width_request: Some(320),
        text_width_chars: DROPDOWN_TEXT_WIDTH_CHARS,
        summary_lines: 2,
        body_lines: 4,
        body_preview_lines: None,
    }
}

pub(crate) fn notification_card(
    notification: &NotificationToast,
    options: NotificationCardOptions,
    callbacks: NotificationCardCallbacks,
) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    root.add_css_class("notification-card");
    root.add_css_class(options.class_name);
    root.add_css_class(notification.urgency_class());

    if let Some(width) = options.width_request {
        root.set_width_request(width);
    }

    let icon = gtk::Image::new();
    icon.add_css_class("notification-card-icon");
    icon.set_valign(gtk::Align::Start);
    crate::notifications::icon::set_notification_icon(&icon, notification);
    root.append(&icon);

    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.add_css_class("notification-card-text");
    text.set_hexpand(true);
    text.append(&message_widget(notification, options, &callbacks));

    if let Some(on_action) = callbacks.on_action {
        let actions = notification.visible_actions();
        if !actions.is_empty() {
            text.append(&actions_widget(notification.id, &actions, on_action));
        }
    }

    root.append(&text);

    if let Some(on_dismiss) = callbacks.on_dismiss {
        root.append(&dismiss_button(notification.id, on_dismiss));
    }

    root
}

fn message_widget(
    notification: &NotificationToast,
    options: NotificationCardOptions,
    callbacks: &NotificationCardCallbacks,
) -> gtk::Widget {
    let message = gtk::Box::new(gtk::Orientation::Vertical, 4);
    message.add_css_class("notification-card-message");

    let app_name = gtk::Label::new(Some(&notification.app_name));
    app_name.add_css_class("notification-card-app-name");
    app_name.set_halign(gtk::Align::Start);
    app_name.set_xalign(0.0);
    app_name.set_max_width_chars(options.text_width_chars);
    app_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    message.append(&app_name);

    let summary = gtk::Label::new(Some(&notification.summary));
    summary.add_css_class("notification-card-summary");
    configure_wrapping_label(&summary, options.text_width_chars, options.summary_lines);
    message.append(&summary);

    if let Some(body) = body_text(notification, options) {
        let label = gtk::Label::new(Some(&body));
        label.add_css_class("notification-card-body");
        configure_wrapping_label(&label, options.text_width_chars, options.body_lines);
        message.append(&label);
    }

    let Some(on_default) = callbacks.on_default.clone() else {
        return message.upcast();
    };

    if !notification.has_default_action() {
        return message.upcast();
    }

    let button = gtk::Button::new();
    button.add_css_class("notification-card-message-button");
    button.add_css_class("flat");
    button.set_child(Some(&message));

    let id = notification.id;
    button.connect_clicked(move |_| {
        on_default(id);
    });

    button.upcast()
}

fn actions_widget(
    notification_id: u32,
    actions: &[super::model::NotificationAction],
    on_action: Rc<dyn Fn(u32, String)>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.add_css_class("notification-card-actions");

    for action in actions {
        let button = gtk::Button::with_label(&action.label);
        button.add_css_class("notification-card-action");
        button.add_css_class("flat");

        let action_id = action.id.clone();
        let on_action = on_action.clone();
        button.connect_clicked(move |_| {
            on_action(notification_id, action_id.clone());
        });

        row.append(&button);
    }

    row
}

fn dismiss_button(notification_id: u32, on_dismiss: Rc<dyn Fn(u32)>) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("notification-card-dismiss");
    button.add_css_class("flat");
    button.set_valign(gtk::Align::Start);
    button.set_icon_name("window-close-symbolic");

    button.connect_clicked(move |_| {
        on_dismiss(notification_id);
    });

    button
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
