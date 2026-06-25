use relm4::{gtk, gtk::prelude::*, prelude::ComponentSender};

use super::super::page::{SettingsConfig, render_section};
use super::super::window::SettingsWindow;
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

    render_actions_section(container, config);

    render_section(
        container,
        super::style_sections::section("Action menu", &config.style),
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

fn render_actions_section(container: &gtk::Box, config: &SettingsConfig) {
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
        for section in &sections {
            section_box.append(&render_section_card(section));
        }
    }

    container.append(&section_box);
}

fn render_section_card(section: &toml::value::Table) -> gtk::Box {
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
        for action in actions {
            if let Some(action) = action.as_table() {
                card.append(&render_action_row(action));
            }
        }
    }

    card
}

fn render_action_row(action: &toml::value::Table) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("settings-row");

    let str_field = |key: &str| action.get(key).and_then(|value| value.as_str());

    let icon = gtk::Label::new(Some(str_field("icon").unwrap_or("")));
    row.append(&icon);

    let label = gtk::Label::new(Some(str_field("label").unwrap_or("(no label)")));
    label.set_halign(gtk::Align::Start);
    label.set_hexpand(true);
    row.append(&label);

    let detail_text = match str_field("action").unwrap_or("command") {
        "open-settings" => "Opens settings".to_string(),
        _ => str_field("command").unwrap_or("").to_string(),
    };
    let detail = gtk::Label::new(Some(&detail_text));
    detail.add_css_class("settings-row-description");
    row.append(&detail);

    row
}