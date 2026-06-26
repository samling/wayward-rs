use crate::settings::window::SettingsWindow;
use crate::settings_spec::{ChoiceSpec, NumberSpec, StringListSpec, StringSpec, ToggleSpec};
use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};

use super::shared::{SettingWriter, append_reset_button, row_with_label};

fn scalar_row_shell(
    label: &str,
    description: Option<&str>,
    path: &'static [&'static str],
    is_configured: bool,
    sender: &ComponentSender<SettingsWindow>,
    align: &gtk::SizeGroup,
) -> (gtk::Box, gtk::Box, SettingWriter, gtk::Button) {
    let (row, controls) = row_with_label(label, description, align);
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let reset_button = append_reset_button(&row, is_configured, writer.clone());

    (row, controls, writer, reset_button)
}

pub(crate) fn number_row(
    setting: NumberSpec,
    sender: &ComponentSender<SettingsWindow>,
    align: &gtk::SizeGroup,
) -> gtk::Box {
    let (row, controls, writer, reset_button) = scalar_row_shell(
        setting.label,
        setting.description,
        setting.path,
        setting.value.is_some(),
        sender,
        align,
    );

    let spin = gtk::SpinButton::with_range(setting.min, setting.max, setting.step);
    spin.set_value(setting.display_value());
    spin.set_width_chars(5);

    let saved_setting = setting.clone();
    let change_writer = writer.clone();

    controls.append(&spin);

    spin.connect_value_changed(move |spin| {
        reset_button.set_sensitive(true);
        change_writer.send_debounced(saved_setting.value_for_config(spin.value()));
    });
    row
}

pub(crate) fn toggle_row(
    setting: ToggleSpec,
    sender: &ComponentSender<SettingsWindow>,
    align: &gtk::SizeGroup,
) -> gtk::Box {
    let (row, controls, writer, reset_button) = scalar_row_shell(
        setting.label,
        setting.description,
        setting.path,
        setting.value.is_some(),
        sender,
        align,
    );

    let toggle = gtk::ToggleButton::with_label(toggle_label(setting.display_value()));
    toggle.add_css_class("settings-toggle-button");
    toggle.set_active(setting.display_value());
    toggle.set_valign(gtk::Align::Center);

    let saved_setting = setting.clone();
    let change_writer = writer.clone();

    controls.append(&toggle);

    toggle.connect_toggled(move |toggle| {
        let active = toggle.is_active();
        toggle.set_label(toggle_label(active));
        reset_button.set_sensitive(true);
        change_writer.send_now(Some(saved_setting.value_for_config(active)));
    });
    row
}

pub(crate) fn choice_row(
    setting: ChoiceSpec,
    sender: &ComponentSender<SettingsWindow>,
    align: &gtk::SizeGroup,
) -> gtk::Box {
    let (row, controls, writer, reset_button) = scalar_row_shell(
        setting.label,
        setting.description,
        setting.path,
        setting.value.is_some(),
        sender,
        align,
    );

    let string_list = gtk::StringList::new(&[]);
    for option in setting.options {
        string_list.append(option.label);
    }
    let dropdown = gtk::DropDown::new(Some(string_list), None::<gtk::Expression>);
    dropdown.add_css_class("settings-choice-dropdown");
    dropdown.set_selected(setting.selected_index());

    let saved_setting = setting.clone();
    let change_writer = writer.clone();

    controls.append(&dropdown);

    dropdown.connect_selected_notify(move |dropdown| {
        reset_button.set_sensitive(true);
        change_writer.send_now(Some(saved_setting.value_for_config(dropdown.selected())));
    });

    row
}

pub(crate) fn string_row(
    setting: StringSpec,
    sender: &ComponentSender<SettingsWindow>,
    align: &gtk::SizeGroup,
) -> gtk::Box {
    let (row, controls, writer, reset_button) = scalar_row_shell(
        setting.label,
        setting.description,
        setting.path,
        setting.value.is_some(),
        sender,
        align,
    );

    let entry = gtk::Entry::new();
    entry.set_text(&setting.display_value());
    entry.set_width_chars(18);

    let saved_setting = setting.clone();
    let change_writer = writer.clone();

    controls.append(&entry);

    entry.connect_changed(move |entry| {
        reset_button.set_sensitive(true);
        change_writer.send_debounced(saved_setting.value_for_config(entry.text().to_string()));
    });
    row
}

pub(crate) fn string_list_row(
    setting: StringListSpec,
    sender: &ComponentSender<SettingsWindow>,
    align: &gtk::SizeGroup,
) -> gtk::Box {
    let (row, controls, writer, reset_button) = scalar_row_shell(
        setting.label,
        setting.description,
        setting.path,
        setting.value.is_some(),
        sender,
        align,
    );

    let entry = gtk::Entry::new();
    entry.set_text(&setting.display_value());
    entry.set_width_chars(28);
    entry.set_tooltip_text(Some("Separate values with commas"));

    let saved_setting = setting.clone();
    let change_writer = writer.clone();

    controls.append(&entry);

    entry.connect_changed(move |entry| {
        reset_button.set_sensitive(true);
        change_writer.send_debounced(saved_setting.value_for_config(entry.text().to_string()));
    });
    row
}

fn toggle_label(active: bool) -> &'static str {
    if active { "On" } else { "Off" }
}
