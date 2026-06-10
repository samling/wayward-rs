use crate::config::{BarConfig, BarRegionKey, StyleConfig};
use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt},
    },
    prelude::*,
};

use super::{
    controls::{display_row, number_row, string_row, toggle_row},
    spec::{SettingSpec, SettingsPageSpec, SettingsSectionSpec},
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

pub(crate) struct SettingsWindow {
    config: SettingsConfig,
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
    SetConfig(SettingsConfig),
    SetValue {
        path: &'static [&'static str],
        value: Option<crate::config::ConfigValue>,
    },
    SetBarRegion {
        bar_name: String,
        region: BarRegionKey,
        widgets: Vec<String>,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BarRegionView {
    Start,
    Center,
    End,
}

impl BarRegionView {
    fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Center => "Center",
            Self::End => "End",
        }
    }

    fn key(self) -> BarRegionKey {
        match self {
            Self::Start => BarRegionKey::Start,
            Self::Center => BarRegionKey::Center,
            Self::End => BarRegionKey::End,
        }
    }

    fn widgets<'a>(self, bar: &'a BarConfig) -> Option<&'a [String]> {
        match self {
            Self::Start => bar.start.as_deref(),
            Self::Center => bar.center.as_deref(),
            Self::End => bar.end.as_deref(),
        }
    }
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
    config: &SettingsConfig,
    sender: &ComponentSender<SettingsWindow>,
) {
    clear_container(container);

    let page = match active_page {
        SettingsPage::Appearance => {
            let page = super::pages::notifications::page(&config.style);
            title.set_label(&page.title);
            render_page(container, page, sender);
        }
        SettingsPage::BarLayout => {
            title.set_label(SettingsPage::BarLayout.title());
            render_bar_layout_page(container, &config.bars, sender);
        }
    };
}

fn bar_region_row(
    bar_name: Option<&str>,
    region: BarRegionView,
    widgets: &[String],
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("settings-row");
    row.add_css_class("bar-region-settings-row");

    let label = gtk::Label::new(Some(region.label()));
    label.set_halign(gtk::Align::Start);
    label.set_width_chars(8);
    label.add_css_class("settings-row-label");
    row.append(&label);

    let values = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    values.set_hexpand(true);
    values.add_css_class("bar-region-widget-list");

    for index in 0..widgets.len() {
        values.append(&bar_widget_token(bar_name, region, widgets, index, sender));
    }

    if widgets.is_empty() {
        let empty = gtk::Label::new(Some("Empty"));
        empty.add_css_class("settings-row-value");
        values.append(&empty);
    }

    row.append(&values);
    row
}

fn bar_widget_token(
    bar_name: Option<&str>,
    region: BarRegionView,
    widgets: &[String],
    index: usize,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let token = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    token.add_css_class("bar-widget-token");

    let label = gtk::Label::new(Some(&widgets[index]));
    label.add_css_class("bar-widget-token-label");
    token.append(&label);

    let move_left = gtk::Button::from_icon_name("go-previous-symbolic");
    move_left.add_css_class("flat");
    move_left.add_css_class("bar-widget-token-button");
    move_left.set_sensitive(bar_name.is_some() && index > 0);
    token.append(&move_left);

    let move_right = gtk::Button::from_icon_name("go-next-symbolic");
    move_right.add_css_class("flat");
    move_right.add_css_class("bar-widget-token-button");
    move_right.set_sensitive(bar_name.is_some() && index + 1 < widgets.len());
    token.append(&move_right);

    let bar_name_left = bar_name.map(str::to_string);
    let widgets_left = widgets.to_vec();
    let sender_left = sender.input_sender().clone();

    move_left.connect_clicked(move |_| {
        let Some(bar_name) = &bar_name_left else {
            return;
        };
        let mut widgets = widgets_left.clone();
        widgets.swap(index - 1, index);

        let _ = sender_left.send(SettingsInput::SetBarRegion {
            bar_name: bar_name.clone(),
            region: region.key(),
            widgets,
        });
    });

    let bar_name_right = bar_name.map(str::to_string);
    let widgets_right = widgets.to_vec();
    let sender_right = sender.input_sender().clone();

    move_right.connect_clicked(move |_| {
        let Some(bar_name) = &bar_name_right else {
            return;
        };

        let mut widgets = widgets_right.clone();
        widgets.swap(index, index + 1);

        let _ = sender_right.send(SettingsInput::SetBarRegion {
            bar_name: bar_name.clone(),
            region: region.key(),
            widgets,
        });
    });

    token
}

fn render_bar_layout_page(
    container: &gtk::Box,
    bars: &[BarConfig],
    sender: &ComponentSender<SettingsWindow>,
) {
    for (index, bar) in bars.iter().enumerate() {
        render_bar_layout_section(container, index, bar, sender);
    }
}

fn render_bar_layout_section(
    container: &gtk::Box,
    index: usize,
    bar: &BarConfig,
    sender: &ComponentSender<SettingsWindow>,
) {
    let section_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    section_box.add_css_class("settings-section");

    let title_text = bar
        .name
        .clone()
        .unwrap_or_else(|| format!("Bar {}", index + 1));

    let title = gtk::Label::new(Some(&title_text));
    title.set_halign(gtk::Align::Start);
    title.add_css_class("settings-section-title");
    section_box.append(&title);

    let group = gtk::Box::new(gtk::Orientation::Vertical, 12);
    group.add_css_class("settings-group");

    for region in [
        BarRegionView::Start,
        BarRegionView::Center,
        BarRegionView::End,
    ] {
        let widgets = region.widgets(bar).unwrap_or(&[]);
        group.append(&bar_region_row(bar.name.as_deref(), region, widgets, sender));
    }

    section_box.append(&group);
    container.append(&section_box);
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
            SettingSpec::Display(setting) => {
                group.append(&display_row(setting));
            }
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

        let page = super::pages::notifications::page(&model.config.style);
        widgets.page_title.set_label(&page.title);
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
        }

        self.update_view(widgets, sender);
    }
}
