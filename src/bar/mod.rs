mod battery;
mod clock;
mod item;
mod layout;
mod registry;
mod workspaces;

use layout::{BarItem, BarLayout};

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;
use relm4::prelude::*;

use crate::workspace::WorkspaceSummary;

pub struct Bar {
    layout: BarLayout,
    pub(super) workspaces: Vec<WorkspaceSummary>,
    pub(super) status: Option<String>,
    pub(super) clock_text: String,
    pub(super) battery_text: String,
}

#[derive(Debug)]
pub enum BarMsg {
    WorkspacesChanged(Vec<WorkspaceSummary>),
    BatteryChanged(String),
    BatteryUnavailable,
    ClockChanged(String),
    NiriUnavailable(String),
    UpdatesStopped,
}

impl Bar {
    fn configure_window(root: &gtk::ApplicationWindow) {
        root.init_layer_shell();
        root.set_layer(Layer::Top);
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);
        root.auto_exclusive_zone_enable();
        root.set_keyboard_mode(KeyboardMode::None);
        root.set_namespace(Some("wayward"));
    }

    fn start_watchers(layout: &BarLayout, sender: &ComponentSender<Self>) {
        for item in layout.items() {
            registry::start_item(item, sender);
        }
    }

    fn initial_model() -> Self {
        Self {
            layout: BarLayout::default_top_bar(),
            workspaces: Vec::new(),
            status: Some("Connecting to Niri".to_string()),
            clock_text: registry::initial_clock_text(),
            battery_text: registry::initial_battery_text(),
        }
    }

    fn render_layout(
        &self,
        left_items: &gtk::Box,
        center_items: &gtk::Box,
        right_items: &gtk::Box,
    ) {
        self.render_region(&self.layout.left, left_items);
        self.render_region(&self.layout.center, center_items);
        self.render_region(&self.layout.right, right_items);
    }

    fn render_region(&self, items: &[BarItem], container: &gtk::Box) {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        for item in items {
            registry::render_item(self, *item, container);
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for Bar {
    type Init = ();
    type Input = BarMsg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Wayward"),
            set_default_height: 32,
            set_resizable: true,

            gtk::CenterBox {
                #[wrap(Some)]
                #[name = "left_region"]
                set_start_widget = &gtk::Box {
                    add_css_class: "bar-region",

                    #[name = "left_items"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 4,
                    }
                },

                #[wrap(Some)]
                #[name = "center_region"]
                set_center_widget = &gtk::Box {
                    set_hexpand: true,
                    set_halign: gtk::Align::Center,
                    add_css_class: "bar-region",

                    #[name = "center_items"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 4,
                    }
                },

                #[wrap(Some)]
                #[name = "right_region"]
                set_end_widget = &gtk::Box {
                    add_css_class: "bar-region",

                    #[name = "right_items"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 4,
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        Self::configure_window(&root);

        let model = Self::initial_model();
        let widgets = view_output!();

        model.render_layout(
            &widgets.left_items,
            &widgets.center_items,
            &widgets.right_items,
        );

        Self::start_watchers(&model.layout, &sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            BarMsg::WorkspacesChanged(workspaces) => {
                self.workspaces = workspaces;
                self.status = None;
            }
            BarMsg::BatteryChanged(battery_text) => {
                self.battery_text = battery_text;
            }
            BarMsg::BatteryUnavailable => {
                self.battery_text = registry::initial_battery_text();
            }
            BarMsg::ClockChanged(clock_text) => {
                self.clock_text = clock_text;
            }
            BarMsg::NiriUnavailable(error) => {
                self.workspaces.clear();
                self.status = Some(format!("Niri unavailable: {error}"));
            }
            BarMsg::UpdatesStopped => {
                self.status = Some("Niri updates stopped".to_string());
            }
        }
    }

    fn pre_view() {
        self.render_layout(&left_items, &center_items, &right_items);
    }
}
