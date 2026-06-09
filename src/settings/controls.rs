use relm4::{gtk::{self, prelude::{BoxExt, EditableExt, WidgetExt}}, prelude::*};
use super::{spec::{NumberSpec, ToggleSpec}, window::SettingsInput};

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
    row
}