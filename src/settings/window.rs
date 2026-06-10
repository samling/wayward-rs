use crate::config::BarRegionKey;
use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt},
    },
    prelude::*,
};

use super::page::{SettingsPage, render_current_page, sidebar_button_classes};

pub(crate) use super::page::SettingsConfig;

pub(crate) struct SettingsWindow {
    config: SettingsConfig,
    active_page: SettingsPage,
}

#[derive(Debug)]
pub(crate) enum SettingsInput {
    SetPage(SettingsPage),
    SetConfig(SettingsConfig),
    SetValue {
        path: &'static [&'static str],
        value: Option<crate::config::ConfigValue>,
    },
    SetBarRegion {
        bar_name: String,
        region: BarRegionKey,
        widgets: Vec<String>,
    },
    AddBar {
        name: String,
    },
    RemoveBar {
        name: String
    },
    SetBarMonitors {
        bar_name: String,
        monitors: Vec<String>,
    },
    SetBarEdge {
        bar_name: String,
        edge: String,
    }
}

#[relm4::component(pub(crate))]
impl Component for SettingsWindow {
    type Init = SettingsConfig;
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

                    #[name = "appearance_button"]
                    gtk::Button {
                        add_css_class: "settings-sidebar-item",

                        #[watch]
                        set_css_classes: &sidebar_button_classes(model.active_page, SettingsPage::Appearance),

                        connect_clicked[sender] => move |_| {
                            sender.input(SettingsInput::SetPage(SettingsPage::Appearance));
                        },

                        gtk::Label {
                            set_label: SettingsPage::Appearance.title(),
                            set_halign: gtk::Align::Start,
                        },
                    },

                    #[name = "bar_layout_button"]
                    gtk::Button {
                        add_css_class: "settings-sidebar-item",

                        #[watch]
                        set_css_classes: &sidebar_button_classes(model.active_page, SettingsPage::BarLayout),

                        connect_clicked[sender] => move |_| {
                            sender.input(SettingsInput::SetPage(SettingsPage::BarLayout));
                        },

                        gtk::Label {
                            set_label: SettingsPage::BarLayout.title(),
                            set_halign: gtk::Align::Start,
                        },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 18,

                    #[name = "page_title"]
                    gtk::Label {
                        set_label: "",
                        set_halign: gtk::Align::Start,
                        add_css_class: "settings-page-title",
                    },

                    #[name = "page_content"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 18,
                    }
                },
            },
        }
    }

    fn init(
        config: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            config,
            active_page: SettingsPage::Appearance,
        };
        let widgets = view_output!();

        render_current_page(
            &widgets.page_content,
            &widgets.page_title,
            model.active_page,
            &model.config,
            &sender,
        );

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
            SettingsInput::SetPage(page) => {
                if self.active_page != page {
                    self.active_page = page;
                    render_current_page(
                        &widgets.page_content,
                        &widgets.page_title,
                        self.active_page,
                        &self.config,
                        &sender,
                    );
                }
            }
            SettingsInput::SetConfig(config) => {
                if self.config != config {
                    self.config = config;
                    render_current_page(
                        &widgets.page_content,
                        &widgets.page_title,
                        self.active_page,
                        &self.config,
                        &sender,
                    );
                }
            }
            SettingsInput::SetValue { path, value } => {
                let value_for_model = value.clone();

                if let Err(error) = crate::config::set_config_value(path, value) {
                    tracing::error!(?path, "Failed to save setting: {error}")
                } else if self
                    .config
                    .style
                    .apply_config_value(path, value_for_model.as_ref())
                    && value_for_model.is_none()
                {
                    render_current_page(
                        &widgets.page_content,
                        &widgets.page_title,
                        self.active_page,
                        &self.config,
                        &sender,
                    );
                }
            }
            SettingsInput::SetBarRegion {
                bar_name,
                region,
                widgets,
            } => {
                if let Err(error) = crate::config::set_bar_region(&bar_name, region, &widgets) {
                    tracing::error!(bar_name, ?region, "Failed to save bar region: {error}")
                }
            }
            SettingsInput::AddBar { name } => {
                if let Err(error) = crate::config::add_bar(&name) {
                    tracing::error!(name, "Failed to add bar: {error}")
                }
            }
            SettingsInput::RemoveBar { name } => {
                if let Err(error) = crate::config::remove_bar(&name) {
                    tracing::error!(name, "Failed to remove bar: {error}")
                }
            }
            SettingsInput::SetBarMonitors { bar_name, monitors } => {
                if let Err(error) = crate::config::set_bar_monitors(&bar_name, &monitors) {
                    tracing::error!(bar_name, ?monitors, "Failed to save bar monitors: {error}");
                }
            }
            SettingsInput::SetBarEdge { bar_name, edge } => {
                if let Err(error) = crate::config::set_bar_edge(&bar_name, &edge) {
                    tracing::error!(bar_name, edge, "Failed to save bar edge: {error}");
                }
            }
        }

        self.update_view(widgets, sender);
    }
}
