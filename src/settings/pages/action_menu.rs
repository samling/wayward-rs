use relm4::{gtk, gtk::prelude::*, prelude::ComponentSender};

use super::super::page::{SettingsConfig, render_section};
use super::super::window::{SettingsInput, SettingsWindow};
use crate::settings_spec::{
    NumberSpec, SettingSpec, SettingsSectionSpec, StringListSpec, StringSpec, table_string,
    table_string_list, table_u16,
};

pub(crate) fn render(
    container: &gtk::Box,
    config: &SettingsConfig,
    sender: &ComponentSender<SettingsWindow>,
) {
    let action_menu = config.widgets.get("action_menu");
    let sub = |key: &str| {
        action_menu
            .and_then(|table| table.get(key))
            .and_then(|value| value.as_table())
    };
    let panel = sub("panel");
    let layout = sub("layout");
    let header = sub("header");

    render_actions_section(container, config, sender);

    render_section(
        container,
        SettingsSectionSpec {
            title: "Panel".to_string(),
            settings: vec![SettingSpec::Number(NumberSpec {
                label: "Width",
                description: None,
                path: &["widgets", "action_menu", "panel", "width"],
                value: panel.and_then(|table| table_u16(table, "width")),
                default: 268,
                min: 120.0,
                max: 1200.0,
                step: 4.0,
            })],
        },
        sender,
    );

    render_section(
        container,
        SettingsSectionSpec {
            title: "Layout".to_string(),
            settings: vec![
                SettingSpec::Number(NumberSpec {
                    label: "Columns",
                    description: None,
                    path: &["widgets", "action_menu", "layout", "columns"],
                    value: layout.and_then(|table| table_u16(table, "columns")),
                    default: 3,
                    min: 1.0,
                    max: 8.0,
                    step: 1.0,
                }),
                SettingSpec::Number(NumberSpec {
                    label: "Button width",
                    description: None,
                    path: &["widgets", "action_menu", "layout", "button-width"],
                    value: layout.and_then(|table| table_u16(table, "button-width")),
                    default: 40,
                    min: 16.0,
                    max: 200.0,
                    step: 2.0,
                }),
                SettingSpec::Number(NumberSpec {
                    label: "Button height",
                    description: None,
                    path: &["widgets", "action_menu", "layout", "button-height"],
                    value: layout.and_then(|table| table_u16(table, "button-height")),
                    default: 40,
                    min: 16.0,
                    max: 200.0,
                    step: 2.0,
                }),
                SettingSpec::Number(NumberSpec {
                    label: "Row spacing",
                    description: None,
                    path: &["widgets", "action_menu", "layout", "row-spacing"],
                    value: layout.and_then(|table| table_u16(table, "row-spacing")),
                    default: 12,
                    min: 0.0,
                    max: 48.0,
                    step: 1.0,
                }),
            ],
        },
        sender,
    );

    render_section(
        container,
        SettingsSectionSpec {
            title: "Header".to_string(),
            settings: vec![
                SettingSpec::String(StringSpec {
                    label: "Power command",
                    description: None,
                    path: &["widgets", "action_menu", "header", "power-command"],
                    value: header.and_then(|table| table_string(table, &["power-command"])),
                    default: "wlogout",
                }),
                SettingSpec::StringList(StringListSpec {
                    label: "Power command args",
                    description: None,
                    path: &["widgets", "action_menu", "header", "power-args"],
                    value: header.and_then(|table| table_string_list(table, "power-args")),
                    default: &[],
                }),
            ],
        },
        sender,
    );

    render_section(
        container,
        super::style_sections::section("Action menu", &config.style),
        sender,
    );
}

fn read_sections(config: &SettingsConfig) -> Vec<toml::value::Table> {
    match config
        .widgets
        .get("action_menu")
        .and_then(|table| table.get("sections"))
    {
        // Configured (including an explicit empty list): show exactly what's set.
        Some(value) => value
            .as_array()
            .map(|sections| {
                sections
                    .iter()
                    .filter_map(|section| section.as_table().cloned())
                    .collect()
            })
            .unwrap_or_default(),
        // Never configured: show the built-in defaults.
        None => crate::bar::widgets::action_menu::default_sections()
            .iter()
            .filter_map(|section| section.as_table().cloned())
            .collect(),
    }
}

fn render_actions_section(
    container: &gtk::Box,
    config: &SettingsConfig,
    sender: &ComponentSender<SettingsWindow>,
) {
    let section_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    section_box.add_css_class("settings-section");

    let title = gtk::Label::new(Some("Buttons"));
    title.set_halign(gtk::Align::Start);
    title.add_css_class("settings-section-title");
    section_box.append(&title);

    let sections = read_sections(config);
    if sections.is_empty() {
        let empty = gtk::Label::new(Some(
            "No sections configured. Add one in your config to edit buttons here.",
        ));
        empty.set_halign(gtk::Align::Start);
        empty.add_css_class("settings-row-description");
        section_box.append(&empty);
    } else {
        for (section_index, section) in sections.iter().enumerate() {
            section_box.append(&render_section_card(section_index, section, sender));
        }
    }

    let add_section = gtk::Button::with_label("Add section");
    add_section.set_halign(gtk::Align::Start);
    let input = sender.input_sender().clone();
    add_section.connect_clicked(move |_| {
        let _ = input.send(SettingsInput::AddActionMenuSection);
    });
    section_box.append(&add_section);

    container.append(&section_box);
}

fn render_section_card(
    section_index: usize,
    section: &toml::value::Table,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
    card.add_css_class("action-menu-settings-section");

    let header_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let title_entry = gtk::Entry::new();
    title_entry.set_hexpand(true);
    title_entry.set_placeholder_text(Some("Section title"));
    title_entry.set_text(
        section
            .get("title")
            .and_then(|value| value.as_str())
            .unwrap_or(""),
    );
    {
        let input = sender.input_sender().clone();
        let entry_for_commit = title_entry.clone();
        let commit = move || {
            let text = entry_for_commit.text().to_string();
            let value = (!text.is_empty()).then_some(crate::config::ConfigValue::String(text));
            let _ = input.send(SettingsInput::SetActionMenuSectionField {
                section: section_index,
                field: "title",
                value,
            });
        };
        let focus = gtk::EventControllerFocus::new();
        focus.connect_leave(move |_| commit());
        title_entry.add_controller(focus);
    }
    header_row.append(&title_entry);

    let columns_label = gtk::Label::new(Some("Columns"));
    header_row.append(&columns_label);
    let columns_spin = gtk::SpinButton::with_range(1.0, 8.0, 1.0);
    columns_spin.set_value(
        section
            .get("columns")
            .and_then(|value| value.as_integer())
            .unwrap_or(3) as f64,
    );
    {
        let input = sender.input_sender().clone();
        columns_spin.connect_value_changed(move |spin| {
            let _ = input.send(SettingsInput::SetActionMenuSectionField {
                section: section_index,
                field: "columns",
                value: Some(crate::config::ConfigValue::Integer(spin.value() as i64)),
            });
        });
    }
    header_row.append(&columns_spin);

    let remove_section = gtk::Button::with_label("Remove section");
    let input = sender.input_sender().clone();
    remove_section.connect_clicked(move |_| {
        let _ = input.send(SettingsInput::RemoveActionMenuSection { section: section_index });
    });
    header_row.append(&remove_section);
    card.append(&header_row);

    if let Some(actions) = section.get("actions").and_then(|value| value.as_array()) {
        for (action_index, action) in actions.iter().enumerate() {
            if let Some(action) = action.as_table() {
                card.append(&render_action_row(section_index, action_index, action, sender));
            }
        }
    }

    let add_action = gtk::Button::with_label("Add button");
    add_action.set_halign(gtk::Align::Start);
    let input = sender.input_sender().clone();
    add_action.connect_clicked(move |_| {
        let _ = input.send(SettingsInput::AddActionMenuAction { section: section_index });
    });
    card.append(&add_action);

    card
}

fn render_action_row(
    section_index: usize,
    action_index: usize,
    action: &toml::value::Table,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 4);
    outer.add_css_class("settings-row");

    let str_field =
        |key: &str| action.get(key).and_then(|value| value.as_str()).unwrap_or("");

    let top = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let icon = action_text_field(section_index, action_index, "icon", str_field("icon"), sender);
    icon.set_width_chars(3);
    icon.set_hexpand(false);
    icon.set_placeholder_text(Some("icon"));
    top.append(&icon);
    let label = action_text_field(section_index, action_index, "label", str_field("label"), sender);
    label.set_placeholder_text(Some("label"));
    top.append(&label);
    top.append(&action_kind_field(section_index, action_index, str_field("action"), sender));
    outer.append(&top);

    let command =
        action_text_field(section_index, action_index, "command", str_field("command"), sender);
    command.set_placeholder_text(Some("command"));
    outer.append(&command);

    let bottom = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let args_text = action
        .get("args")
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    bottom.append(&action_args_field(section_index, action_index, &args_text, sender));
    let show_label = action
        .get("show-label")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    bottom.append(&action_toggle_field(section_index, action_index, show_label, sender));

    let remove = gtk::Button::with_label("Remove");
    let input_remove = sender.input_sender().clone();
    remove.connect_clicked(move |_| {
        let _ = input_remove.send(SettingsInput::RemoveActionMenuAction {
            section: section_index,
            action: action_index,
        });
    });
    bottom.append(&remove);

    outer.append(&bottom);

    outer
}

fn action_text_field(
    section_index: usize,
    action_index: usize,
    field: &'static str,
    current: &str,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Entry {
    let entry = gtk::Entry::new();
    entry.set_text(current);
    entry.set_hexpand(true);

    let input = sender.input_sender().clone();
    let entry_for_commit = entry.clone();
    let commit = move || {
        let text = entry_for_commit.text().to_string();
        let value = (!text.is_empty()).then_some(crate::config::ConfigValue::String(text));
        let _ = input.send(SettingsInput::SetActionMenuActionField {
            section: section_index,
            action: action_index,
            field,
            value,
        });
    };

    let focus = gtk::EventControllerFocus::new();
    focus.connect_leave(move |_| commit());
    entry.add_controller(focus);

    entry
}

fn action_kind_field(
    section_index: usize,
    action_index: usize,
    current: &str,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::DropDown {
    let options = ["command", "open-settings"];
    let string_list = gtk::StringList::new(&["Command", "Open settings"]);
    let dropdown = gtk::DropDown::new(Some(string_list), None::<gtk::Expression>);
    let selected = options.iter().position(|option| *option == current).unwrap_or(0) as u32;
    dropdown.set_selected(selected);

    let input = sender.input_sender().clone();
    dropdown.connect_selected_notify(move |dropdown| {
        let value = options.get(dropdown.selected() as usize).copied().unwrap_or("command");
        let _ = input.send(SettingsInput::SetActionMenuActionField {
            section: section_index,
            action: action_index,
            field: "action",
            value: Some(crate::config::ConfigValue::String(value.to_string())),
        });
    });

    dropdown
}

fn action_toggle_field(
    section_index: usize,
    action_index: usize,
    current: bool,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::CheckButton {
    let check = gtk::CheckButton::with_label("Show label");
    check.set_active(current);

    let input = sender.input_sender().clone();
    check.connect_toggled(move |check| {
        let _ = input.send(SettingsInput::SetActionMenuActionField {
            section: section_index,
            action: action_index,
            field: "show-label",
            value: Some(crate::config::ConfigValue::Bool(check.is_active())),
        });
    });

    check
}

fn action_args_field(
    section_index: usize,
    action_index: usize,
    current: &str,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Entry {
    let entry = gtk::Entry::new();
    entry.set_text(current);
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("args (space separated)"));

    let input = sender.input_sender().clone();
    let entry_for_commit = entry.clone();
    let commit = move || {
        let args: Vec<String> = entry_for_commit
            .text()
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect();
        let value = (!args.is_empty()).then_some(crate::config::ConfigValue::StringList(args));
        let _ = input.send(SettingsInput::SetActionMenuActionField {
            section: section_index,
            action: action_index,
            field: "args",
            value,
        });
    };

    let focus = gtk::EventControllerFocus::new();
    focus.connect_leave(move |_| commit());
    entry.add_controller(focus);

    entry
}