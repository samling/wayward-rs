use crate::config::StyleConfig;
use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, GtkWindowExt, OrientableExt, WidgetExt},
    },
    prelude::*,
};

use super::{
    controls::{number_row, string_row, toggle_row},
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
    },
}

fn clear_container(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn render_current_page(
    container: &gtk::Box,
    title: &gtk::Label,
    style: &StyleConfig,
    sender: &ComponentSender<SettingsWindow>,
) {
    clear_container(container);

    let page = super::pages::notifications::page(style);
    title.set_label(page.title);
    render_page(container, page, sender);
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

#[relm4::component(pub(crate))]
impl Component for SettingsWindow {
    type Init = StyleConfig;
    type Input = SettingsInput;
    type Output = ();
    type CommandOutput = ();

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

                        #[name = "page_title"]
                        gtk::Label {
                            set_label: "",
                            set_halign: gtk::Align::Start,
                            add_css_class: "settings-page-title",
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

        let page = super::pages::notifications::page(&model.style);
        widgets.page_title.set_label(page.title);
        render_current_page(&widgets.page_content, &widgets.page_title, &model.style, &sender);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            SettingsInput::SetStyle(style) => {
                if self.style != style {
                    self.style = style;
                    render_current_page(
                        &widgets.page_content,
                        &widgets.page_title,
                        &self.style,
                        &sender,
                    );
                }
            }
            SettingsInput::SetValue { path, value } => {
                let value_for_model = value.clone();

                if let Err(error) = crate::config::set_config_value(path, value) {
                    tracing::error!(?path, "Failed to save setting: {error}")
                } else if self
                    .style
                    .apply_config_value(path, value_for_model.as_ref())
                    && value_for_model.is_none()
                {
                    render_current_page(&widgets.page_content, &widgets.page_title, &self.style, &sender);
                }
            }
        }

        self.update_view(widgets, sender);
    }
}
