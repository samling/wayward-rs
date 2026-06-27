use crate::config::{BarConfig, StyleConfig};
use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, ButtonExt, WidgetExt},
    },
    prelude::ComponentSender,
};
use std::collections::BTreeMap;

use super::{
    controls::{choice_row, color_row, number_row, string_list_row, string_row, toggle_row},
    window::{SettingsInput, SettingsWindow},
};
use crate::settings_spec::{SettingSpec, SettingsPageSpec, SettingsSectionSpec};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SettingsConfig {
    pub(crate) style: StyleConfig,
    pub(crate) widgets: BTreeMap<String, toml::value::Table>,
    pub(crate) bars: Vec<BarConfig>,
    pub(crate) available_monitors: Vec<String>,
}

impl SettingsConfig {
    pub(crate) fn new(config: &crate::config::AppConfig, available_monitors: Vec<String>) -> Self {
        Self {
            style: config.style.clone(),
            widgets: config.widgets.clone(),
            bars: config.bars.clone(),
            available_monitors,
        }
    }

    pub(crate) fn apply_config_value(
        &mut self,
        path: &[&str],
        value: Option<&crate::config::ConfigValue>,
    ) -> bool {
        if self.style.apply_config_value(path, value) {
            return true;
        }

        let ["widgets", widget, rest @ ..] = path else {
            return false;
        };

        match value {
            Some(value) => {
                let table = self.widgets.entry((*widget).to_string()).or_default();
                insert_widget_value(table, rest, config_value_to_toml(value));
            }
            None => {
                if let Some(table) = self.widgets.get_mut(*widget) {
                    remove_widget_value(table, rest);
                    if table.is_empty() {
                        self.widgets.remove(*widget);
                    }
                }
            }
        }

        true
    }
}

fn insert_widget_value(table: &mut toml::value::Table, path: &[&str], value: toml::Value) {
    let Some((key, rest)) = path.split_first() else {
        return;
    };

    if rest.is_empty() {
        table.insert((*key).to_string(), value);
        return;
    }

    let child = table
        .entry((*key).to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));

    if !child.is_table() {
        *child = toml::Value::Table(toml::value::Table::new());
    }

    if let Some(child_table) = child.as_table_mut() {
        insert_widget_value(child_table, rest, value);
    }
}

fn remove_widget_value(table: &mut toml::value::Table, path: &[&str]) -> bool {
    let Some((key, rest)) = path.split_first() else {
        return table.is_empty();
    };

    if rest.is_empty() {
        table.remove(*key);
    } else if let Some(child) = table.get_mut(*key).and_then(toml::Value::as_table_mut) {
        if remove_widget_value(child, rest) {
            table.remove(*key);
        }
    }

    table.is_empty()
}

fn config_value_to_toml(value: &crate::config::ConfigValue) -> toml::Value {
    match value {
        crate::config::ConfigValue::Bool(value) => toml::Value::Boolean(*value),
        crate::config::ConfigValue::Integer(value) => toml::Value::Integer(*value),
        crate::config::ConfigValue::String(value) => toml::Value::String(value.clone()),
        crate::config::ConfigValue::StringList(values) => toml::Value::Array(
            values
                .iter()
                .map(|value| toml::Value::String(value.clone()))
                .collect(),
        ),
    }
}

pub(crate) fn render_current_page(
    container: &gtk::Box,
    title: &gtk::Label,
    active_item: &'static str,
    config: &SettingsConfig,
    sender: &ComponentSender<SettingsWindow>,
    action_menu_editor: &mut Option<
        relm4::Controller<super::pages::action_menu_editor::ActionMenuEditor>,
    >,
) {
    *action_menu_editor = None;
    clear_container(container);

    let Some(item) = super::nav::find_item(active_item) else {
        return;
    };

    match &item.content {
        super::nav::NavContent::BarLayout => {
            title.set_label(item.title);
            super::pages::bar_layout::render(
                container,
                &config.bars,
                &config.available_monitors,
                sender,
            );
        }
        super::nav::NavContent::ActionMenu => {
            title.set_label(item.title);
            super::pages::action_menu::render(container, config, sender, action_menu_editor);
        }
        _ => {
            if let Some(page) = build_page(item, config) {
                title.set_label(&page.title);
                render_page(container, page, sender);
            }
        }
    }
}

pub(crate) fn render_sidebar(
    container: &gtk::Box,
    active_item: &str,
    sender: &ComponentSender<SettingsWindow>,
) {
    clear_container(container);

    for group in super::nav::nav() {
        let header = gtk::Label::new(Some(group.title));
        header.set_halign(gtk::Align::Start);
        header.add_css_class("settings-sidebar-group");
        container.append(&header);

        for item in group.items {
            let button = gtk::Button::new();
            let classes: &[&str] = if item.key == active_item {
                &["settings-sidebar-item", "active"]
            } else {
                &["settings-sidebar-item"]
            };
            button.set_css_classes(classes);

            let label = gtk::Label::new(Some(item.title));
            label.set_halign(gtk::Align::Start);
            button.set_child(Some(&label));

            let key = item.key;
            let input_sender = sender.input_sender().clone();
            button.connect_clicked(move |_| {
                let _ = input_sender.send(SettingsInput::SelectNavItem(key));
            });

            container.append(&button);
        }
    }
}

fn clear_container(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn render_page(
    container: &gtk::Box,
    page: SettingsPageSpec,
    sender: &ComponentSender<SettingsWindow>,
) {
    for section in page.sections {
        render_section(container, section, sender);
    }
}

pub(super) fn render_section(
    container: &gtk::Box,
    section: SettingsSectionSpec,
    sender: &ComponentSender<SettingsWindow>,
) {
    let section_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    section_box.add_css_class("settings-section");

    let title = gtk::Label::new(Some(&section.title));
    title.set_halign(gtk::Align::Start);
    title.add_css_class("settings-section-title");
    section_box.append(&title);

    let group = gtk::Box::new(gtk::Orientation::Vertical, 12);
    group.add_css_class("settings-group");

    // Shared column so every row's control cluster has the same width and aligns.
    let controls_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
    // Shared width for the color value control, so the opacity column past it
    // does not shift with the selected palette name's length.
    let color_value_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);

    for setting in section.settings {
        match setting {
            SettingSpec::Number(setting) => {
                group.append(&number_row(setting, sender, &controls_group));
            }
            SettingSpec::Toggle(setting) => {
                group.append(&toggle_row(setting, sender, &controls_group));
            }
            SettingSpec::String(setting) => {
                group.append(&string_row(setting, sender, &controls_group));
            }
            SettingSpec::StringList(setting) => {
                group.append(&string_list_row(setting, sender, &controls_group));
            }
            SettingSpec::Choice(setting) => {
                group.append(&choice_row(setting, sender, &controls_group));
            }
            SettingSpec::Color(setting) => {
                group.append(&color_row(
                    setting,
                    sender,
                    &controls_group,
                    &color_value_group,
                ));
            }
        }
    }

    section_box.append(&group);
    container.append(&section_box);
}

pub(crate) fn build_page(
    item: &super::nav::NavItem,
    config: &SettingsConfig,
) -> Option<SettingsPageSpec> {
    use super::nav::NavContent;

    match &item.content {
        NavContent::StyleSection(section) => Some(SettingsPageSpec {
            title: item.title.to_string(),
            sections: vec![super::pages::style_sections::section(
                section,
                &config.style,
            )],
        }),
        NavContent::Widget {
            section,
            config_key,
        } => {
            let mut sections = super::pages::widgets::config_sections(config_key, &config.widgets);
            let mut style = super::pages::style_sections::section(section, &config.style);
            style.title = "Style".to_string();
            sections.push(style);
            Some(SettingsPageSpec {
                title: item.title.to_string(),
                sections,
            })
        }
        NavContent::BarLayout => None,
        NavContent::ActionMenu => None,
    }
}

#[cfg(test)]
#[path = "page_test.rs"]
mod tests;
