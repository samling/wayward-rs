use super::{
    spec::{ColorSpec, NumberSpec, StringListSpec, StringSpec, ToggleSpec},
    window::SettingsInput,
};
use crate::config::ConfigValue;
use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

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

    let toggle = gtk::ToggleButton::with_label(toggle_label(setting.display_value()));
    toggle.add_css_class("settings-toggle-button");
    toggle.set_active(setting.display_value());
    toggle.set_valign(gtk::Align::Center);

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();

    row.append(&toggle);
    let reset_button = append_reset_button(&row, setting.value.is_some(), writer);

    toggle.connect_toggled(move |toggle| {
        let active = toggle.is_active();
        toggle.set_label(toggle_label(active));
        reset_button.set_sensitive(true);
        change_writer.send_now(Some(saved_setting.value_for_config(active)));
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

pub(crate) fn string_list_row(
    setting: StringListSpec,
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
    entry.set_width_chars(28);
    entry.set_tooltip_text(Some("Separate values with commas"));

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
    let (button, swatch, swatch_color) = color_swatch_button(color);
    button.set_valign(gtk::Align::Center);

    let entry = gtk::Entry::new();
    entry.add_css_class("settings-color-value");
    entry.set_text(&css_color_value(color));
    entry.set_width_chars(24);

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();
    let updating = Rc::new(Cell::new(false));

    row.append(&button);
    row.append(&entry);
    let reset_button = append_reset_button(&row, setting.value.is_some(), writer);

    let entry_swatch = swatch.clone();
    let entry_swatch_color = swatch_color.clone();
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
        set_swatch_color(&entry_swatch, &entry_swatch_color, color);
        entry_updating.set(false);

        entry_writer.send_debounced(entry_saved_setting.value_for_config(css_color_value(color)));
    });

    let button_entry = entry.clone();
    let button_reset_button = reset_button.clone();
    let button_updating = updating.clone();
    let button_swatch = swatch.clone();
    let button_swatch_color = swatch_color.clone();

    button.connect_clicked(move |button| {
        let initial_color = swatch_color.borrow().to_owned();
        let parent = button
            .root()
            .and_then(|root| root.downcast::<gtk::Window>().ok());

        let button_entry = button_entry.clone();
        let button_reset_button = button_reset_button.clone();
        let button_updating = button_updating.clone();
        let button_swatch = button_swatch.clone();
        let button_swatch_color = button_swatch_color.clone();
        let change_writer = change_writer.clone();
        let saved_setting = saved_setting.clone();

        dialog.choose_rgba(
            parent.as_ref(),
            Some(&initial_color),
            None::<&gtk::gio::Cancellable>,
            move |result| {
                let Ok(color) = result else {
                    return;
                };

                let value = css_color_value(color);

                button_updating.set(true);
                set_swatch_color(&button_swatch, &button_swatch_color, color);
                button_entry.set_text(&value);
                button_updating.set(false);

                button_reset_button.set_sensitive(true);
                change_writer.send_now(Some(saved_setting.value_for_config(value)));
            },
        );
    });

    row
}

fn color_swatch_button(
    color: gtk::gdk::RGBA,
) -> (gtk::Button, gtk::DrawingArea, Rc<RefCell<gtk::gdk::RGBA>>) {
    let button = gtk::Button::new();
    button.add_css_class("settings-color-swatch-button");
    button.set_tooltip_text(Some("Pick color"));

    let swatch = gtk::DrawingArea::new();
    swatch.add_css_class("settings-color-swatch");
    swatch.set_content_width(28);
    swatch.set_content_height(18);

    let swatch_color = Rc::new(RefCell::new(color));
    let draw_color = swatch_color.clone();

    swatch.set_draw_func(move |_, context, width, height| {
        let color = draw_color.borrow();
        context.set_source_rgba(
            color.red().into(),
            color.green().into(),
            color.blue().into(),
            color.alpha().into(),
        );
        context.rectangle(0.0, 0.0, width as f64, height as f64);
        let _ = context.fill();
    });

    button.set_child(Some(&swatch));

    (button, swatch, swatch_color)
}

fn set_swatch_color(
    swatch: &gtk::DrawingArea,
    swatch_color: &Rc<RefCell<gtk::gdk::RGBA>>,
    color: gtk::gdk::RGBA,
) {
    swatch_color.replace(color);
    swatch.queue_draw();
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

fn toggle_label(active: bool) -> &'static str {
    if active { "On" } else { "Off" }
}
