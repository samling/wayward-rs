use super::{
    spec::{ColorSettingRole, ColorSpec, NumberSpec, StringListSpec, StringSpec, ToggleSpec},
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

#[derive(Clone)]
struct OwnedSettingWriter {
    path: Vec<String>,
    input_sender: relm4::Sender<SettingsInput>,
    pending: Rc<RefCell<Option<gtk::glib::SourceId>>>,
}

impl OwnedSettingWriter {
    fn new(path: Vec<String>, input_sender: relm4::Sender<SettingsInput>) -> Self {
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

        let _ = self.input_sender.send(SettingsInput::SetValueOwned {
            path: self.path.clone(),
            value,
        });
    }

    fn cancel_pending(&self) {
        if let Some(source) = self.pending.borrow_mut().take() {
            source.remove();
        }
    }
}

/// Builds a settings row with a leading label and a controls box that shares a
/// width column (via `align`) so clusters line up across rows. Append value
/// widgets to the returned controls box, then the reset button to the row.
fn row_with_label(label_text: &str, align: &gtk::SizeGroup) -> (gtk::Box, gtk::Box) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.add_css_class("settings-row");

    let label = gtk::Label::new(Some(label_text));
    label.set_hexpand(true);
    label.set_halign(gtk::Align::Start);
    label.add_css_class("settings-row-label");
    row.append(&label);

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.set_halign(gtk::Align::Start);
    controls.set_valign(gtk::Align::Center);
    align.add_widget(&controls);
    row.append(&controls);

    (row, controls)
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
    align: &gtk::SizeGroup,
) -> gtk::Box {
    let (row, controls) = row_with_label(setting.label, align);

    let spin = gtk::SpinButton::with_range(setting.min, setting.max, setting.step);
    spin.set_value(setting.display_value());
    spin.set_width_chars(5);

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();

    controls.append(&spin);
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
    align: &gtk::SizeGroup,
) -> gtk::Box {
    let (row, controls) = row_with_label(setting.label, align);

    let toggle = gtk::ToggleButton::with_label(toggle_label(setting.display_value()));
    toggle.add_css_class("settings-toggle-button");
    toggle.set_active(setting.display_value());
    toggle.set_valign(gtk::Align::Center);

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();

    controls.append(&toggle);
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
    align: &gtk::SizeGroup,
) -> gtk::Box {
    let (row, controls) = row_with_label(setting.label, align);

    let entry = gtk::Entry::new();
    entry.set_text(&setting.display_value());
    entry.set_width_chars(18);

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();

    controls.append(&entry);
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
    align: &gtk::SizeGroup,
) -> gtk::Box {
    let (row, controls) = row_with_label(setting.label, align);

    let entry = gtk::Entry::new();
    entry.set_text(&setting.display_value());
    entry.set_width_chars(28);
    entry.set_tooltip_text(Some("Separate values with commas"));

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();

    controls.append(&entry);
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
    align: &gtk::SizeGroup,
    value_align: &gtk::SizeGroup,
) -> gtk::Box {
    if setting.role == ColorSettingRole::Palette {
        return palette_color_row(setting, sender, align);
    }
    consumer_color_row(setting, sender, align, value_align)
}

fn palette_color_row(
    setting: ColorSpec,
    sender: &ComponentSender<super::window::SettingsWindow>,
    align: &gtk::SizeGroup,
) -> gtk::Box {
    let label_text = setting.display_label();
    let (row, controls) = row_with_label(&label_text, align);

    let color = parse_color(&setting.display_value());
    let dialog = gtk::ColorDialog::builder()
        .title(&label_text)
        .modal(true)
        .with_alpha(false)
        .build();
    let (button, swatch, swatch_color) = color_swatch_button(color);
    button.set_valign(gtk::Align::Center);

    let entry = gtk::Entry::new();
    entry.add_css_class("settings-color-value");
    if setting.is_custom() {
        entry.add_css_class("settings-color-value-custom");
    }
    entry.set_text(&setting.entry_value());
    entry.set_width_chars(24);

    let path = setting.path;
    let saved_setting = setting.clone();
    let writer = SettingWriter::new(path, sender.input_sender().clone());
    let change_writer = writer.clone();
    let updating = Rc::new(Cell::new(false));

    controls.append(&button);
    controls.append(&entry);
    let reset_button = append_reset_button(&row, setting.value.is_some(), writer);
    reset_button.set_tooltip_text(Some(setting.reset_tooltip()));

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
        entry.add_css_class("settings-color-value-custom");
        entry_reset_button.set_sensitive(true);

        entry_updating.set(true);
        set_swatch_color(&entry_swatch, &entry_swatch_color, color);
        entry_updating.set(false);

        let hex = solid_hex(color);
        entry_writer.send_debounced(entry_saved_setting.value_for_config(hex));
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

                let value = solid_hex(color);

                button_updating.set(true);
                set_swatch_color(&button_swatch, &button_swatch_color, color);
                button_entry.set_text(&value);
                button_updating.set(false);

                button_entry.add_css_class("settings-color-value-custom");
                button_reset_button.set_sensitive(true);
                change_writer.send_now(Some(saved_setting.value_for_config(value)));
            },
        );
    });

    row
}

fn consumer_color_row(
    setting: ColorSpec,
    sender: &ComponentSender<super::window::SettingsWindow>,
    align: &gtk::SizeGroup,
    value_align: &gtk::SizeGroup,
) -> gtk::Box {
    let label_text = setting.display_label();
    let (row, controls) = row_with_label(&label_text, align);

    // --- toggle ---
    let palette_toggle = gtk::ToggleButton::with_label("Palette");
    let custom_toggle = gtk::ToggleButton::with_label("Custom");
    custom_toggle.set_group(Some(&palette_toggle));
    palette_toggle.set_active(setting.is_palette_ref);
    custom_toggle.set_active(!setting.is_palette_ref);

    let toggle_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    toggle_box.add_css_class("linked");
    toggle_box.append(&palette_toggle);
    toggle_box.append(&custom_toggle);
    toggle_box.set_valign(gtk::Align::Center);

    // --- palette page ---
    let palette_options = Rc::new(setting.palette_options.clone());
    let string_list = gtk::StringList::new(&[]);
    for opt in palette_options.iter() {
        string_list.append(&opt.label);
    }
    let dropdown = gtk::DropDown::new(Some(string_list), None::<gtk::Expression>);
    dropdown.set_width_request(150);

    let effective_token = setting
        .value
        .clone()
        .or_else(|| setting.default_token.map(str::to_string));
    let selected_idx = effective_token
        .and_then(|tok| palette_options.iter().position(|o| o.token == tok))
        .unwrap_or(0) as u32;
    dropdown.set_selected(selected_idx);

    let palette_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    palette_box.append(&dropdown);

    // --- custom page ---
    // In palette mode the custom box should mirror the currently-selected token's
    // color, so toggling to Custom shows the live color rather than the inherited one.
    let initial_custom = if setting.is_palette_ref {
        palette_options
            .get(selected_idx as usize)
            .map(|o| o.color.clone())
            .unwrap_or_else(|| setting.display_value())
    } else {
        setting.display_value()
    };
    let custom_color = parse_color(&initial_custom);
    let custom_dialog = gtk::ColorDialog::builder()
        .title(&label_text)
        .modal(true)
        .with_alpha(false)
        .build();
    let (custom_button, custom_swatch, custom_swatch_color) = color_swatch_button(custom_color);
    custom_button.set_valign(gtk::Align::Center);

    let custom_entry = gtk::Entry::new();
    custom_entry.add_css_class("settings-color-value");
    custom_entry.set_placeholder_text(setting.placeholder());
    let entry_text = if setting.is_palette_ref {
        solid_hex(custom_color)
    } else {
        setting.entry_value()
    };
    custom_entry.set_text(&entry_text);
    custom_entry.set_width_chars(18);

    let custom_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    custom_box.append(&custom_entry);

    // --- stack ---
    let stack = gtk::Stack::new();
    stack.add_named(&palette_box, Some("palette"));
    stack.add_named(&custom_box, Some("custom"));
    stack.set_visible_child_name(if setting.is_palette_ref { "palette" } else { "custom" });
    value_align.add_widget(&stack);

    // --- opacity field ---
    let opacity_scale = gtk::SpinButton::with_range(0.0, 100.0, 1.0);
    opacity_scale.set_digits(0);
    opacity_scale.set_numeric(true);
    opacity_scale.set_value(setting.display_opacity() as f64);
    opacity_scale.set_valign(gtk::Align::Center);
    opacity_scale.set_tooltip_text(Some("Opacity (%)"));

    // --- writers ---
    let color_writer = SettingWriter::new(setting.path, sender.input_sender().clone());
    let opacity_writer = OwnedSettingWriter::new(
        setting.opacity_path.clone(),
        sender.input_sender().clone(),
    );

    let opacity_label = gtk::Label::new(Some("Opacity"));
    opacity_label.add_css_class("settings-opacity-label");
    opacity_label.set_valign(gtk::Align::Center);

    controls.append(&toggle_box);
    controls.append(&custom_button);
    controls.append(&stack);
    controls.append(&opacity_label);
    controls.append(&opacity_scale);

    let is_configured = setting.value.is_some() || setting.opacity.is_some();
    let reset_color_writer = color_writer.clone();
    let reset_opacity_writer = opacity_writer.clone();

    let reset_button = {
        let button = gtk::Button::with_label("Reset");
        button.add_css_class("settings-reset-button");
        button.set_tooltip_text(Some(setting.reset_tooltip()));
        button.set_sensitive(is_configured);
        row.append(&button);
        button
    };

    // --- toggle wiring ---
    let toggle_stack = stack.clone();
    palette_toggle.connect_toggled(move |btn| {
        if btn.is_active() {
            toggle_stack.set_visible_child_name("palette");
        } else {
            toggle_stack.set_visible_child_name("custom");
        }
    });

    // Guards swatch/entry writes so programmatic updates don't echo back to config.
    let entry_updating = Rc::new(Cell::new(false));

    // --- dropdown wiring ---
    let dd_options = palette_options.clone();
    let dd_writer = color_writer.clone();
    let dd_reset = reset_button.clone();
    let dd_swatch = custom_swatch.clone();
    let dd_swatch_color = custom_swatch_color.clone();
    let dd_entry = custom_entry.clone();
    let dd_updating = entry_updating.clone();
    dropdown.connect_selected_notify(move |dd| {
        let idx = dd.selected() as usize;
        if let Some(opt) = dd_options.get(idx) {
            dd_reset.set_sensitive(true);
            dd_writer.send_now(Some(ConfigValue::String(opt.token.to_string())));

            // Mirror the selected token's color into the custom box so switching
            // to Custom shows the live color, not the build-time inherited one.
            let color = parse_color(&opt.color);
            dd_updating.set(true);
            set_swatch_color(&dd_swatch, &dd_swatch_color, color);
            dd_entry.set_text(&solid_hex(color));
            dd_updating.set(false);
        }
    });

    // --- custom entry wiring ---
    let entry_swatch = custom_swatch.clone();
    let entry_swatch_color = custom_swatch_color.clone();
    let entry_reset = reset_button.clone();
    let entry_writer = color_writer.clone();
    let entry_updating2 = entry_updating.clone();
    let entry_saved_setting = setting.clone();

    custom_entry.connect_changed(move |entry| {
        if entry_updating2.get() {
            return;
        }

        let value = entry.text().to_string();
        let Ok(color) = gtk::gdk::RGBA::parse(&value) else {
            entry.add_css_class("error");
            entry_reset.set_sensitive(true);
            return;
        };

        entry.remove_css_class("error");
        entry_reset.set_sensitive(true);

        entry_updating2.set(true);
        set_swatch_color(&entry_swatch, &entry_swatch_color, color);
        entry_updating2.set(false);

        entry_writer.send_debounced(entry_saved_setting.value_for_config(solid_hex(color)));
    });

    // --- custom button wiring ---
    let btn_entry = custom_entry.clone();
    let btn_reset = reset_button.clone();
    let btn_updating = entry_updating.clone();
    let btn_swatch = custom_swatch.clone();
    let btn_swatch_color = custom_swatch_color.clone();
    let btn_writer = color_writer.clone();
    let btn_saved_setting = setting.clone();
    let btn_custom_toggle = custom_toggle.clone();

    custom_button.connect_clicked(move |button| {
        let initial = btn_swatch_color.borrow().to_owned();
        let parent = button
            .root()
            .and_then(|root| root.downcast::<gtk::Window>().ok());

        let btn_entry = btn_entry.clone();
        let btn_reset = btn_reset.clone();
        let btn_updating = btn_updating.clone();
        let btn_swatch = btn_swatch.clone();
        let btn_swatch_color = btn_swatch_color.clone();
        let btn_writer = btn_writer.clone();
        let btn_saved_setting = btn_saved_setting.clone();
        let btn_custom_toggle = btn_custom_toggle.clone();

        custom_dialog.choose_rgba(
            parent.as_ref(),
            Some(&initial),
            None::<&gtk::gio::Cancellable>,
            move |result| {
                let Ok(color) = result else {
                    return;
                };

                let value = solid_hex(color);

                btn_updating.set(true);
                set_swatch_color(&btn_swatch, &btn_swatch_color, color);
                btn_entry.set_text(&value);
                btn_updating.set(false);

                btn_reset.set_sensitive(true);
                // Picking a literal color makes this a custom value, not a palette ref.
                btn_custom_toggle.set_active(true);
                btn_writer.send_now(Some(btn_saved_setting.value_for_config(value)));
            },
        );
    });

    // --- opacity wiring ---
    debug_assert!(!setting.opacity_path.is_empty(), "consumer color {:?} has empty opacity_path", setting.path);
    let opacity_reset = reset_button.clone();
    let opacity_writer2 = opacity_writer.clone();
    opacity_scale.connect_value_changed(move |scale| {
        opacity_reset.set_sensitive(true);
        opacity_writer2.send_debounced(ConfigValue::Integer(scale.value() as i64));
    });

    // --- reset wiring ---
    reset_button.connect_clicked(move |_| {
        reset_color_writer.send_now(None);
        reset_opacity_writer.send_now(None);
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

fn solid_hex(color: gtk::gdk::RGBA) -> String {
    let r = (color.red() * 255.0).round() as u8;
    let g = (color.green() * 255.0).round() as u8;
    let b = (color.blue() * 255.0).round() as u8;
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn toggle_label(active: bool) -> &'static str {
    if active { "On" } else { "Off" }
}
