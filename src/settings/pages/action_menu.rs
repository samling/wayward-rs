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
        super::style_sections::section("Action menu", &config.style),
        sender,
    );

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
    config
        .widgets
        .get("action_menu")
        .and_then(|table| table.get("sections"))
        .and_then(|value| value.as_array())
        .map(|sections| {
            sections
                .iter()
                .filter_map(|section| section.as_table().cloned())
                .collect()
        })
        .unwrap_or_default()
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

    container.append(&section_box);
}

fn render_section_card(
    section_index: usize,
    section: &toml::value::Table,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
    card.add_css_class("action-menu-settings-section");

    let title = section
        .get("title")
        .and_then(|value| value.as_str())
        .unwrap_or("(untitled section)");
    let header_text = match section.get("columns").and_then(|value| value.as_integer()) {
        Some(columns) => format!("{title} - {columns} columns"),
        None => title.to_string(),
    };

    let header = gtk::Label::new(Some(&header_text));
    header.set_halign(gtk::Align::Start);
    header.add_css_class("settings-row-label");
    card.append(&header);

    if let Some(actions) = section.get("actions").and_then(|value| value.as_array()) {
        for (action_index, action) in actions.iter().enumerate() {
            if let Some(action) = action.as_table() {
                card.append(&render_action_row(section_index, action_index, action, sender));
            }
        }
    }

    card
}

fn render_action_row(
    section_index: usize,
    action_index: usize,
    action: &toml::value::Table,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("settings-row");

    let str_field =
        |key: &str| action.get(key).and_then(|value| value.as_str()).unwrap_or("");

    let icon = action_text_field(section_index, action_index, "icon", str_field("icon"), sender);
    icon.set_width_chars(3);
    icon.set_hexpand(false);
    icon.set_placeholder_text(Some("icon"));
    row.append(&icon);

    let label = action_text_field(section_index, action_index, "label", str_field("label"), sender);
    label.set_placeholder_text(Some("label"));
    row.append(&label);

    let command =
        action_text_field(section_index, action_index, "command", str_field("command"), sender);
    command.set_placeholder_text(Some("command"));
    row.append(&command);

    row
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