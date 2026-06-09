use crate::config::ConfigValue;
use std::{cell::RefCell, rc::Rc, time::Duration};
use super::{
    spec::{NumberSpec, StringSpec, ToggleSpec},
    window::SettingsInput,
};
use relm4::{
    gtk::{
        self,
        prelude::*,
    },
    prelude::*,
};

const SETTING_WRITE_DEBOUNCE: Duration = Duration::from_millis(300);

#[derive(Clone)]
struct SettingWriter {
    path: &'static [&'static str],
    input_sender: relm4::Sender<SettingsInput>,
    pending: Rc<RefCell<Option<gtk::glib::SourceId>>>,
}

impl SettingWriter {
    fn new(
        path: &'static [&'static str],
        input_sender: relm4::Sender<SettingsInput>,
    ) -> Self {
        Self {
            path,
            input_sender,
            pending: Rc::new(RefCell::new(None)),
        }
    }

    fn send_debounced(&self, value: ConfigValue) {
        self.cancel_pending();

        let writer = self.clone();
        let source = gtk::glib::timeout_add_local_once(SETTING_WRITE_DEBOUNCE, move || {
            *writer.pending.borrow_mut() = None;
            writer.send_now(Some(value));
        });

        *self.pending.borrow_mut() = Some(source);
    }

    fn send_now(&self, value: Option<ConfigValue>) {
        self.cancel_pending();

        let _ = self.input_sender.send(SettingsInput::SetValue {
            path: self.path,
            value,
        });
    }

    fn cancel_pending(&self) {
        if let Some(source) = self.pending.borrow_mut().take() {
            source.remove();
        }
    }
}

fn append_reset_button(
    row: &gtk::Box,
    is_configured: bool,
    writer: SettingWriter,
) {
    let button = gtk::Button::from_icon_name("edit-undo-symbolic");
    button.add_css_class("settings-reset-button");
    button.set_tooltip_text(Some("Reset to default"));
    button.set_sensitive(is_configured);

    button.connect_clicked(move |_| {
        writer.send_now(None);
    });

    row.append(&button);
}

pub(crate) fn number_row(
    setting: NumberSpec,
    sender: &ComponentSender<super::window::SettingsWindow>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.add_css_class("settings-row");

    let label = gtk::Label::new(Some(setting.label));
    label.set_hexpand(true);
    label.set_halign(gtk::Align::Start);
    label.add_css_class("settings-row-label");
    row.append(&label);

    let spin = gtk::SpinButton::with_range(setting.min, setting.max, setting.step);
    spin.set_value(setting.display_value());
    spin.set_width_chars(5);

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();

    spin.connect_value_changed(move |spin| {
        change_writer.send_debounced(saved_setting.value_for_config(spin.value()));
    });

    row.append(&spin);
    append_reset_button(&row, setting.value.is_some(), writer);
    row
}

pub(crate) fn toggle_row(
    setting: ToggleSpec,
    sender: &ComponentSender<super::window::SettingsWindow>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.add_css_class("settings-row");

    let label = gtk::Label::new(Some(setting.label));
    label.set_hexpand(true);
    label.set_halign(gtk::Align::Start);
    label.add_css_class("settings-row-label");
    row.append(&label);

    let toggle = gtk::Switch::new();
    toggle.set_active(setting.display_value());
    toggle.set_valign(gtk::Align::Center);

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();

    toggle.connect_active_notify(move |toggle| {
        change_writer.send_debounced(saved_setting.value_for_config(toggle.is_active()));
    });

    row.append(&toggle);
    append_reset_button(&row, setting.value.is_some(), writer);
    row
}

pub(crate) fn string_row(
    setting: StringSpec,
    sender: &ComponentSender<super::window::SettingsWindow>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.add_css_class("settings-row");

    let label = gtk::Label::new(Some(setting.label));
    label.set_hexpand(true);
    label.set_halign(gtk::Align::Start);
    label.add_css_class("settings-row-label");
    row.append(&label);

    let entry = gtk::Entry::new();
    entry.set_text(&setting.display_value());
    entry.set_width_chars(18);

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();

    entry.connect_changed(move |entry| {
        change_writer.send_debounced(saved_setting.value_for_config(entry.text().to_string()));
    });

    row.append(&entry);
    append_reset_button(&row, setting.value.is_some(), writer);
    row
}
