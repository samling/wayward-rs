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
    controls::{color_row, number_row, string_list_row, string_row, toggle_row},
    spec::{SettingSpec, SettingsPageSpec, SettingsSectionSpec},
    window::{SettingsInput, SettingsWindow},
};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsPage {
    Appearance,
    Widgets,
    BarLayout,
}

impl SettingsPage {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Widgets => "Widgets",
            Self::BarLayout => "Bar Layout",
        }
    }
}

pub(crate) fn render_current_page(
    container: &gtk::Box,
    appearance_subpages: &gtk::Box,
    title: &gtk::Label,
    active_page: SettingsPage,
    active_appearance_section: &'static str,
    config: &SettingsConfig,
    sender: &ComponentSender<SettingsWindow>,
) {
    clear_container(container);
    clear_container(appearance_subpages);

    match active_page {
        SettingsPage::Appearance => {
            appearance_subpages.set_visible(true);
            render_appearance_subpages(appearance_subpages, active_appearance_section, sender);
            title.set_label(SettingsPage::Appearance.title());
            render_section(
                container,
                super::pages::notifications::section_spec(active_appearance_section, &config.style),
                sender,
            );
        }
        SettingsPage::Widgets => {
            appearance_subpages.set_visible(false);
            let page = super::pages::widgets::page(&config.widgets);
            title.set_label(&page.title);
            render_page(container, page, sender);
        }
        SettingsPage::BarLayout => {
            appearance_subpages.set_visible(false);
            title.set_label(SettingsPage::BarLayout.title());
            super::pages::bar_layout::render(
                container,
                &config.bars,
                &config.available_monitors,
                sender,
            );
        }
    };
}

pub(crate) fn default_appearance_section() -> &'static str {
    super::pages::notifications::sections()
        .into_iter()
        .next()
        .unwrap_or("Palette")
}

pub(crate) fn sidebar_button_classes(
    active_page: SettingsPage,
    page: SettingsPage,
) -> Vec<&'static str> {
    if active_page == page {
        vec!["settings-sidebar-item", "active"]
    } else {
        vec!["settings-sidebar-item"]
    }
}

fn clear_container(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn render_appearance_subpages(
    container: &gtk::Box,
    active_section: &'static str,
    sender: &ComponentSender<SettingsWindow>,
) {
    for section in super::pages::notifications::sections() {
        let button = gtk::Button::with_label(section);
        button.set_css_classes(&appearance_subpage_button_classes(active_section, section));

        let input_sender = sender.input_sender().clone();
        button.connect_clicked(move |_| {
            let _ = input_sender.send(SettingsInput::SetAppearanceSection(section));
        });

        container.append(&button);
    }
}

fn appearance_subpage_button_classes(
    active_section: &'static str,
    section: &'static str,
) -> Vec<&'static str> {
    if active_section == section {
        vec!["settings-subpage-button", "active"]
    } else {
        vec!["settings-subpage-button"]
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

fn render_section(
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

    for setting in section.settings {
        match setting {
            SettingSpec::Number(setting) => {
                group.append(&number_row(setting, sender));
            }
            SettingSpec::Toggle(setting) => {
                group.append(&toggle_row(setting, sender));
            }
            SettingSpec::String(setting) => {
                group.append(&string_row(setting, sender));
            }
            SettingSpec::StringList(setting) => {
                group.append(&string_list_row(setting, sender));
            }
            SettingSpec::Color(setting) => {
                group.append(&color_row(setting, sender));
            }
        }
    }

    section_box.append(&group);
    container.append(&section_box);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigValue;

    #[test]
    fn settings_config_applies_widget_string_value() {
        let mut config = SettingsConfig {
            style: StyleConfig::default(),
            widgets: BTreeMap::new(),
            bars: Vec::new(),
            available_monitors: Vec::new(),
        };

        assert!(config.apply_config_value(
            &["widgets", "example", "label"],
            Some(&ConfigValue::String("Hello".to_string())),
        ));

        let value = config
            .widgets
            .get("example")
            .and_then(|table| table.get("label"))
            .and_then(|value| value.as_str());

        assert_eq!(value, Some("Hello"));
    }

    #[test]
    fn settings_config_applies_widget_string_list_value() {
        let mut config = SettingsConfig {
            style: StyleConfig::default(),
            widgets: BTreeMap::new(),
            bars: Vec::new(),
            available_monitors: Vec::new(),
        };

        assert!(config.apply_config_value(
            &["widgets", "updates", "critical-patterns"],
            Some(&ConfigValue::StringList(vec!["linux-*".to_string()])),
        ));

        let values = config
            .widgets
            .get("updates")
            .and_then(|table| table.get("critical-patterns"))
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .collect::<Vec<_>>()
            });

        assert_eq!(values, Some(vec!["linux-*"]));
    }

    #[test]
    fn settings_config_removes_empty_widget_table_after_reset() {
        let mut updates = toml::value::Table::new();
        updates.insert(
            "critical-patterns".to_string(),
            toml::Value::Array(vec![toml::Value::String("linux-*".to_string())]),
        );

        let mut config = SettingsConfig {
            style: StyleConfig::default(),
            widgets: BTreeMap::from([("updates".to_string(), updates)]),
            bars: Vec::new(),
            available_monitors: Vec::new(),
        };

        assert!(config.apply_config_value(&["widgets", "updates", "critical-patterns"], None));

        assert!(!config.widgets.contains_key("updates"));
    }

    #[test]
    fn settings_config_applies_nested_widget_string_value() {
        let mut config = SettingsConfig {
            style: StyleConfig::default(),
            widgets: BTreeMap::new(),
            bars: Vec::new(),
            available_monitors: Vec::new(),
        };

        assert!(config.apply_config_value(
            &["widgets", "brightness", "sunsetr", "paused-preset"],
            Some(&ConfigValue::String("day".to_string())),
        ));

        let value = config
            .widgets
            .get("brightness")
            .and_then(|table| table.get("sunsetr"))
            .and_then(toml::Value::as_table)
            .and_then(|table| table.get("paused-preset"))
            .and_then(toml::Value::as_str);

        assert_eq!(value, Some("day"));
    }

    #[test]
    fn settings_config_removes_empty_nested_widget_table_after_reset() {
        let mut sunsetr = toml::value::Table::new();
        sunsetr.insert(
            "paused-preset".to_string(),
            toml::Value::String("day".to_string()),
        );

        let mut brightness = toml::value::Table::new();
        brightness.insert("sunsetr".to_string(), toml::Value::Table(sunsetr));

        let mut config = SettingsConfig {
            style: StyleConfig::default(),
            widgets: BTreeMap::from([("brightness".to_string(), brightness)]),
            bars: Vec::new(),
            available_monitors: Vec::new(),
        };

        assert!(
            config
                .apply_config_value(&["widgets", "brightness", "sunsetr", "paused-preset"], None,)
        );

        assert!(!config.widgets.contains_key("brightness"));
    }
}
