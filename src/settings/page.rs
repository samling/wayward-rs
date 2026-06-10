use crate::config::{BarConfig, StyleConfig};
use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, WidgetExt},
    },
    prelude::ComponentSender,
};

use super::{
    controls::{number_row, string_row, toggle_row},
    spec::{SettingSpec, SettingsPageSpec, SettingsSectionSpec},
    window::SettingsWindow,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SettingsConfig {
    pub(crate) style: StyleConfig,
    pub(crate) bars: Vec<BarConfig>,
}

impl From<&crate::config::AppConfig> for SettingsConfig {
    fn from(config: &crate::config::AppConfig) -> Self {
        Self {
            style: config.style.clone(),
            bars: config.bars.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsPage {
    Appearance,
    BarLayout,
}

impl SettingsPage {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::BarLayout => "Bar Layout",
        }
    }
}

pub(crate) fn render_current_page(
    container: &gtk::Box,
    title: &gtk::Label,
    active_page: SettingsPage,
    config: &SettingsConfig,
    sender: &ComponentSender<SettingsWindow>,
) {
    clear_container(container);

    match active_page {
        SettingsPage::Appearance => {
            let page = super::pages::notifications::page(&config.style);
            title.set_label(&page.title);
            render_page(container, page, sender);
        }
        SettingsPage::BarLayout => {
            title.set_label(SettingsPage::BarLayout.title());
            super::pages::bar_layout::render(container, &config.bars, sender);
        }
    };
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
        }
    }

    section_box.append(&group);
    container.append(&section_box);
}
