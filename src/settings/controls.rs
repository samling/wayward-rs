use super::{
    spec::{ColorSpec, NumberSpec, StringSpec, ToggleSpec},
    window::SettingsInput,
};
use crate::config::ConfigValue;
use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};
use std::{cell::{Cell, RefCell}, rc::Rc, time::Duration};

const SETTING_WRITE_DEBOUNCE: Duration = Duration::from_millis(300);

#[derive(Clone)]
struct SettingWriter {
    path: &'static [&'static str],
    input_sender: relm4::Sender<SettingsInput>,
    pending: Rc<RefCell<Option<gtk::glib::SourceId>>>,
}

impl SettingWriter {
    fn new(path: &'static [&'static str], input_sender: relm4::Sender<SettingsInput>) -> Self {
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

fn append_reset_button(row: &gtk::Box, is_configured: bool, writer: SettingWriter) -> gtk::Button {
    let button = gtk::Button::from_icon_name("edit-undo-symbolic");
    button.add_css_class("settings-reset-button");
    button.set_tooltip_text(Some("Reset to default"));
    button.set_sensitive(is_configured);

    button.connect_clicked(move |_| {
        writer.send_now(None);
    });

    row.append(&button);
    button
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

    row.append(&spin);
    let reset_button = append_reset_button(&row, setting.value.is_some(), writer);

    spin.connect_value_changed(move |spin| {
        reset_button.set_sensitive(true);
        change_writer.send_debounced(saved_setting.value_for_config(spin.value()));
    });
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

    row.append(&toggle);
    let reset_button = append_reset_button(&row, setting.value.is_some(), writer);

    toggle.connect_active_notify(move |toggle| {
        reset_button.set_sensitive(true);
        change_writer.send_now(Some(saved_setting.value_for_config(toggle.is_active())));
    });
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

    row.append(&entry);
    let reset_button = append_reset_button(&row, setting.value.is_some(), writer);

    entry.connect_changed(move |entry| {
        reset_button.set_sensitive(true);
        change_writer.send_debounced(saved_setting.value_for_config(entry.text().to_string()));
    });
    row
}

pub(crate) fn color_row(
    setting: ColorSpec,
    sender: &ComponentSender<super::window::SettingsWindow>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.add_css_class("settings-row");

    let label = gtk::Label::new(Some(setting.label));
    label.set_hexpand(true);
    label.set_halign(gtk::Align::Start);
    label.add_css_class("settings-row-label");
    row.append(&label);

    let color = parse_color(&setting.display_value());
    let dialog = gtk::ColorDialog::builder()
        .title(setting.label)
        .modal(true)
        .with_alpha(true)
        .build();
    let button = gtk::ColorDialogButton::new(Some(dialog));
    button.set_rgba(&color);
    button.set_valign(gtk::Align::Center);

    let entry = gtk::Entry::new();
    entry.add_css_class("settings-color-value");
    entry.set_text(&button.rgba().to_string());
    entry.set_width_chars(24);

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();
    let updating = Rc::new(Cell::new(false));

    row.append(&button);
    row.append(&entry);
    let reset_button = append_reset_button(&row, setting.value.is_some(), writer);

    let entry_button = button.clone();
    let entry_reset_button = reset_button.clone();
    let entry_saved_setting = saved_setting.clone();
    let entry_writer = change_writer.clone();
    let entry_updating = updating.clone();

    entry.connect_changed(move |entry| {
        if entry_updating.get() {
            return;
        }

        let value = entry.text().to_string();
        let Ok(color) = gtk::gdk::RGBA::parse(&value) else {
            entry.add_css_class("error");
            entry_reset_button.set_sensitive(true);
            return;
        };

        entry.remove_css_class("error");
        entry_reset_button.set_sensitive(true);

        entry_updating.set(true);
        entry_button.set_rgba(&color);
        entry_updating.set(false);

        entry_writer.send_debounced(entry_saved_setting.value_for_config(css_color_value(color)));
    });

    let button_entry = entry.clone();
    let button_reset_button = reset_button.clone();
    let button_updating = updating.clone();

    button.connect_rgba_notify(move |button| {
        if button_updating.get() {
            return;
        }

        let value = css_color_value(button.rgba());

        button_updating.set(true);
        button_entry.set_text(&value);
        button_updating.set(false);

        button_reset_button.set_sensitive(true);
        change_writer.send_now(Some(saved_setting.value_for_config(value)));
    });

    row
}

fn parse_color(value: &str) -> gtk::gdk::RGBA {
    gtk::gdk::RGBA::parse(value).unwrap_or(gtk::gdk::RGBA::BLACK)
}

fn css_color_value(color: gtk::gdk::RGBA) -> String {
    let red = (color.red() * 255.0).round() as u8;
    let green = (color.green() * 255.0).round() as u8;
    let blue = (color.blue() * 255.0).round() as u8;
    let alpha = color.alpha();

    if alpha >= 1.0 {
        format!("#{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("rgba({red}, {green}, {blue}, {alpha:.3})")
    }
}