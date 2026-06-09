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

fn append_reset_button(
    row: &gtk::Box,
    path: &'static [&'static str],
    is_configured: bool,
    sender: &ComponentSender<super::window::SettingsWindow>,
) {
    let button = gtk::Button::from_icon_name("edit-undo-symbolic");
    button.add_css_class("settings-reset-button");
    button.set_tooltip_text(Some("Reset to default"));
    button.set_sensitive(is_configured);

    let input_sender = sender.input_sender().clone();

    button.connect_clicked(move |_| {
        let _ = input_sender.send(SettingsInput::SetValue { path, value: None });
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
    let input_sender = sender.input_sender().clone();

    spin.connect_value_changed(move |spin| {
        let _ = input_sender.send(SettingsInput::SetValue {
            path,
            value: Some(saved_setting.value_for_config(spin.value())),
        });
    });

    row.append(&spin);
    append_reset_button(&row, path, setting.value.is_some(), sender);
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
    let input_sender = sender.input_sender().clone();

    toggle.connect_active_notify(move |toggle| {
        let _ = input_sender.send(SettingsInput::SetValue {
            path,
            value: Some(saved_setting.value_for_config(toggle.is_active())),
        });
    });

    row.append(&toggle);
    append_reset_button(&row, path, setting.value.is_some(), sender);
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
    let input_sender = sender.input_sender().clone();

    entry.connect_changed(move |entry| {
        let _ = input_sender.send(SettingsInput::SetValue {
            path,
            value: Some(saved_setting.value_for_config(entry.text().to_string())),
        });
    });

    row.append(&entry);
    append_reset_button(&row, path, setting.value.is_some(), sender);
    row
}
