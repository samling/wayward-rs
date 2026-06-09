use crate::config::StyleConfig;
use relm4::{gtk::{self, prelude::{BoxExt, GtkWindowExt, OrientableExt, WidgetExt}}, prelude::*};

use super::{
    controls::number_row,
    spec::{SettingSpec, SettingsPageSpec, SettingsSectionSpec},
};

pub(crate) struct SettingsWindow {
    style: StyleConfig,
}

#[derive(Debug)]
pub(crate) enum SettingsInput {
    SetStyle(StyleConfig),
    SetValue {
        path: &'static [&'static str],
        value: Option<crate::config::ConfigValue>,
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

    let title = gtk::Label::new(Some(section.title));
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
        }
    }

    section_box.append(&group);
    container.append(&section_box);
}

#[relm4::component(pub(crate))]
impl SimpleComponent for SettingsWindow {
    type Init = StyleConfig;
    type Input = SettingsInput;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Wayward Settings"),
            set_default_size: (900, 650),
            set_hide_on_close: true,
            add_css_class: "settings-window",

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_width_request: 220,
                    add_css_class: "settings-sidebar",

                    gtk::Label {
                        set_label: "Settings",
                        set_halign: gtk::Align::Start,
                        add_css_class: "settings-sidebar-title",
                    },

                    gtk::Button {
                        add_css_class: "settings-sidebar-item",
                        set_sensitive: false,

                        gtk::Label {
                            set_label: "Notifications",
                            set_halign: gtk::Align::Start,
                        },
                    },
                },

                #[name = "page_content"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 18,
                },
            },
        }
    }

    fn init(
        style: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { style };
        let widgets = view_output!();

        render_page(
            &widgets.page_content,
            super::pages::notifications::page(&model.style),
            &sender,
        );

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SettingsInput::SetStyle(style) => {
                self.style = style;
            }
            SettingsInput::SetValue { path, value } => {
                if let Err(error) = crate::config::set_config_value(
                    path,
                    value,
                ) {
                    tracing::error!(?path, "Failed to save setting: {error}")
                }
            }
        }
    }
}