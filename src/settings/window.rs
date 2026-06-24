use crate::config::BarRegionKey;
use relm4::{
    gtk::{
        self,
        prelude::{
            AdjustmentExt, BoxExt, ButtonExt, Cast, EventControllerExt, GestureSingleExt,
            GtkWindowExt, NativeExt, OrientableExt, ToplevelExt, WidgetExt,
        },
    },
    prelude::*,
};

use super::page::{render_current_page, render_sidebar};

pub(crate) use super::page::SettingsConfig;

pub(crate) struct SettingsWindow {
    config: SettingsConfig,
    active_item: &'static str,
    scroll: std::collections::HashMap<&'static str, f64>,
}

#[derive(Debug)]
pub(crate) enum SettingsInput {
    Close,
    SelectNavItem(&'static str),
    SetConfig(SettingsConfig),
    SetValue {
        path: &'static [&'static str],
        value: Option<crate::config::ConfigValue>,
    },
    SetValueOwned {
        path: Vec<String>,
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
        name: String,
    },
    SetBarMonitors {
        bar_name: String,
        monitors: Vec<String>,
    },
    SetBarEdge {
        bar_name: String,
        edge: String,
    },
    RenameBar {
        current_name: String,
        next_name: String,
    },
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
                set_orientation: gtk::Orientation::Vertical,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    add_css_class: "settings-titlebar",

                    #[name = "titlebar_drag_area"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_hexpand: true,
                        add_css_class: "settings-titlebar-drag-area",

                        gtk::Label {
                            set_label: "Wayward Settings",
                            set_halign: gtk::Align::Start,
                            set_valign: gtk::Align::Center,
                            add_css_class: "settings-titlebar-title",
                        },
                    },

                    gtk::Button {
                        set_label: "Close",
                        set_valign: gtk::Align::Center,
                        add_css_class: "settings-titlebar-close",

                        connect_clicked[sender] => move |_| {
                            sender.input(SettingsInput::Close);
                        },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_vexpand: true,

                    #[name = "sidebar"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_width_request: 220,
                        add_css_class: "settings-sidebar",
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 18,
                        set_hexpand: true,
                        set_vexpand: true,

                        #[name = "page_title"]
                        gtk::Label {
                            set_label: "",
                            set_halign: gtk::Align::Start,
                            add_css_class: "settings-page-title",
                        },

                        #[name = "page_scroll"]
                        gtk::ScrolledWindow {
                            set_hexpand: true,
                            set_vexpand: true,
                            set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                            add_css_class: "settings-page-scroll",

                            #[name = "page_content"]
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 18,
                            }
                        }
                    },
                },
            },
        }
    }

    fn init(
        config: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            config,
            active_item: super::nav::DEFAULT_ITEM,
            scroll: std::collections::HashMap::new(),
        };
        let widgets = view_output!();

        install_titlebar_drag(&widgets.titlebar_drag_area, &root);

        render_sidebar(&widgets.sidebar, model.active_item, &sender);
        render_current_page(
            &widgets.page_content,
            &widgets.page_title,
            model.active_item,
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
        root: &Self::Root,
    ) {
        match msg {
            SettingsInput::Close => {
                root.set_visible(false);
            }
            SettingsInput::SelectNavItem(key) => {
                if self.active_item != key {
                    self.save_scroll(widgets);
                    self.active_item = key;
                    render_sidebar(&widgets.sidebar, self.active_item, &sender);
                    render_current_page(
                        &widgets.page_content,
                        &widgets.page_title,
                        self.active_item,
                        &self.config,
                        &sender,
                    );
                    self.restore_scroll(widgets);
                }
            }
            SettingsInput::SetConfig(config) => {
                if self.config != config {
                    self.save_scroll(widgets);
                    self.config = config;
                    render_current_page(
                        &widgets.page_content,
                        &widgets.page_title,
                        self.active_item,
                        &self.config,
                        &sender,
                    );
                    self.restore_scroll(widgets);
                }
            }
            SettingsInput::SetValue { path, value } => {
                let value_for_model = value.clone();

                if let Err(error) = crate::config::set_config_value(path, value) {
                    tracing::error!(?path, "Failed to save setting: {error}")
                } else if self
                    .config
                    .apply_config_value(path, value_for_model.as_ref())
                {
                    if value_for_model.is_none() {
                        self.save_scroll(widgets);
                        render_current_page(
                            &widgets.page_content,
                            &widgets.page_title,
                            self.active_item,
                            &self.config,
                            &sender,
                        );
                        self.restore_scroll(widgets);
                    }
                }
            }
            SettingsInput::SetValueOwned { path, value } => {
                let path_refs: Vec<&str> = path.iter().map(String::as_str).collect();
                let value_for_model = value.clone();

                if let Err(error) = crate::config::set_config_value(&path_refs, value) {
                    tracing::error!(?path, "Failed to save setting: {error}")
                } else if self
                    .config
                    .apply_config_value(&path_refs, value_for_model.as_ref())
                {
                    if value_for_model.is_none() {
                        self.save_scroll(widgets);
                        render_current_page(
                            &widgets.page_content,
                            &widgets.page_title,
                            self.active_item,
                            &self.config,
                            &sender,
                        );
                        self.restore_scroll(widgets);
                    }
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
            SettingsInput::RenameBar {
                current_name,
                next_name,
            } => {
                if let Err(error) = crate::config::rename_bar(&current_name, &next_name) {
                    tracing::error!(current_name, next_name, "Failed to rename bar: {error}");
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl SettingsWindow {
    fn save_scroll(&mut self, widgets: &SettingsWindowWidgets) {
        let value = scroll_page_value(&widgets.page_scroll);
        self.scroll.insert(self.active_item, value);
    }

    fn restore_scroll(&self, widgets: &SettingsWindowWidgets) {
        let value = self.scroll.get(self.active_item).copied().unwrap_or(0.0);
        restore_page_scroll(&widgets.page_scroll, value);
    }
}

fn scroll_page_value(scroller: &gtk::ScrolledWindow) -> f64 {
    let adjustment = scroller.vadjustment();
    adjustment.value()
}

fn restore_page_scroll(scroller: &gtk::ScrolledWindow, value: f64) {
    let adjustment = scroller.vadjustment();

    gtk::glib::idle_add_local_once(move || {
        let lower = adjustment.lower();
        let max = (adjustment.upper() - adjustment.page_size()).max(lower);
        adjustment.set_value(value.clamp(lower, max));
    });
}

fn install_titlebar_drag(drag_area: &gtk::Box, window: &gtk::Window) {
    let click = gtk::GestureClick::builder().button(1).build();
    let window = window.clone();

    click.connect_pressed(move |gesture, _click_count, x, y| {
        let Some(device) = gesture.current_event_device() else {
            return;
        };

        let Some(surface) = window.surface() else {
            return;
        };

        let Ok(toplevel) = surface.downcast::<gtk::gdk::Toplevel>() else {
            return;
        };

        toplevel.begin_move(
            &device,
            gesture.current_button() as i32,
            x,
            y,
            gesture.current_event_time(),
        );
    });

    drag_area.add_controller(click);
}
