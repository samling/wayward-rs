use super::super::window::SettingsInput;
use crate::config::ConfigValue;
use relm4::gtk::{self, prelude::*};
use std::{cell::RefCell, rc::Rc, time::Duration};

const SETTING_WRITE_DEBOUNCE: Duration = Duration::from_millis(300);

#[derive(Clone)]
enum SettingPath {
    Static(&'static [&'static str]),
    Owned(Vec<String>),
}

#[derive(Clone)]
pub(super) struct SettingWriter {
    path: SettingPath,
    input_sender: relm4::Sender<SettingsInput>,
    pending: Rc<RefCell<Option<gtk::glib::SourceId>>>,
}

impl SettingWriter {
    pub(super) fn new(
        path: &'static [&'static str],
        input_sender: relm4::Sender<SettingsInput>,
    ) -> Self {
        Self::from_path(SettingPath::Static(path), input_sender)
    }

    pub(super) fn new_owned(path: Vec<String>, input_sender: relm4::Sender<SettingsInput>) -> Self {
        Self::from_path(SettingPath::Owned(path), input_sender)
    }

    fn from_path(path: SettingPath, input_sender: relm4::Sender<SettingsInput>) -> Self {
        Self {
            path,
            input_sender,
            pending: Rc::new(RefCell::new(None)),
        }
    }

    pub(super) fn send_debounced(&self, value: ConfigValue) {
        self.cancel_pending();

        let writer = self.clone();
        let source = gtk::glib::timeout_add_local_once(SETTING_WRITE_DEBOUNCE, move || {
            *writer.pending.borrow_mut() = None;
            writer.send_now(Some(value));
        });

        *self.pending.borrow_mut() = Some(source);
    }

    pub(super) fn send_now(&self, value: Option<ConfigValue>) {
        self.cancel_pending();

        let message = match &self.path {
            SettingPath::Static(path) => SettingsInput::SetValue { path: *path, value },
            SettingPath::Owned(path) => SettingsInput::SetValueOwned {
                path: path.clone(),
                value,
            },
        };

        let _ = self.input_sender.send(message);
    }

    fn cancel_pending(&self) {
        if let Some(source) = self.pending.borrow_mut().take() {
            source.remove();
        }
    }
}

/// Builds a settings row with a leading label and aligned controls.
/// Append value widgets to the controls box, then append reset to the row.
pub(super) fn row_with_label(
    label_text: &str,
    description: Option<&str>,
    align: &gtk::SizeGroup,
) -> (gtk::Box, gtk::Box) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.add_css_class("settings-row");

    let label_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    label_box.set_hexpand(true);
    label_box.set_halign(gtk::Align::Start);
    label_box.set_valign(gtk::Align::Center);

    let label = gtk::Label::new(Some(label_text));
    label.set_halign(gtk::Align::Start);
    label.add_css_class("settings-row-label");
    row.append(&label);

    if let Some(description) = description {
        let description_label = gtk::Label::new(None);
        description_label.set_markup(description);
        description_label.set_halign(gtk::Align::Start);
        description_label.set_xalign(0.0);
        description_label.set_wrap(true);
        description_label.add_css_class("settings-row-description");
        label_box.append(&description_label);
    }

    row.append(&label_box);

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.set_halign(gtk::Align::Start);
    controls.set_valign(gtk::Align::Center);
    align.add_widget(&controls);
    row.append(&controls);

    (row, controls)
}

pub(super) fn append_reset_button(
    row: &gtk::Box,
    is_configured: bool,
    writer: SettingWriter,
) -> gtk::Button {
    let button = gtk::Button::with_label("Reset");
    button.add_css_class("settings-reset-button");
    button.set_tooltip_text(Some("Reset to default"));
    button.set_sensitive(is_configured);

    button.connect_clicked(move |_| {
        writer.send_now(None);
    });

    row.append(&button);
    button
}
