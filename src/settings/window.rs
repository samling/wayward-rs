use crate::config::StyleConfig;
use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt},
    },
    prelude::*,
};

use super::{
    controls::{number_row, string_row, toggle_row},
    spec::{SettingSpec, SettingsPageSpec, SettingsSectionSpec},
};

pub(crate) struct SettingsWindow {
    style: StyleConfig,
    active_page: SettingsPage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsPage {
    Appearance,
    BarLayout,
}

impl SettingsPage {
    fn title(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::BarLayout => "Bar Layout",
        }
    }
}

#[derive(Debug)]
pub(crate) enum SettingsInput {
    SetPage(SettingsPage),
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
    active_page: SettingsPage,
    style: &StyleConfig,
    sender: &ComponentSender<SettingsWindow>,
) {
    clear_container(container);

    let page = match active_page {
        SettingsPage::Appearance => super::pages::notifications::page(style),
        SettingsPage::BarLayout => bar_layout_page(),
    };

    title.set_label(page.title);
    render_page(container, page, sender);
}

fn bar_layout_page() -> SettingsPageSpec {
    SettingsPageSpec {
        title: "Bar Layout",
        sections: vec![SettingsSectionSpec {
            title: "Widgets",
            settings: Vec::new(),
        }],
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

fn sidebar_button_classes(active_page: SettingsPage, page: SettingsPage) -> Vec<&'static str> {
    if active_page == page {
        vec!["settings-sidebar-item", "active"]
    } else {
        vec!["settings-sidebar-item"]
    }
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
        style: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            style,
            active_page: SettingsPage::Appearance,
        };
        let widgets = view_output!();

        let page = super::pages::notifications::page(&model.style);
        widgets.page_title.set_label(page.title);
        render_current_page(
            &widgets.page_content,
            &widgets.page_title,
            model.active_page,
            &model.style,
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
                        &self.style,
                        &sender,
                    );
                }
            }
            SettingsInput::SetStyle(style) => {
                if self.style != style {
                    self.style = style;
                    render_current_page(
                        &widgets.page_content,
                        &widgets.page_title,
                        self.active_page,
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
                    render_current_page(
                        &widgets.page_content,
                        &widgets.page_title,
                        self.active_page,
                        &self.style,
                        &sender,
                    );
                }
            }
        }

        self.update_view(widgets, sender);
    }
}
