mod battery;
mod clock;
mod item;
mod layout;
mod registry;
mod workspaces;

use layout::{BarEdge, BarItem, BarLayout};

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;
use relm4::prelude::*;

use crate::config::AppConfig;
use crate::workspace::WorkspaceSummary;

pub struct Bar {
    name: Option<String>,
    layout: BarLayout,
    edge: BarEdge,
    active_watchers: Vec<(BarItem, relm4::JoinHandle<()>)>,
    pub(super) workspaces: Vec<WorkspaceSummary>,
    pub(super) status: Option<String>,
    pub(super) clock_text: String,
    pub(super) battery_text: String,
}

pub struct BarInit {
    pub(crate) name: Option<String>,
    pub(crate) layout: BarLayout,
    pub(crate) edge: BarEdge,
}

impl BarInit {
    pub(crate) fn from_config(config: Option<&crate::config::BarConfig>) -> Self {
        Self {
            name: config.and_then(|bar| bar.name.clone()),
            layout: BarLayout::from_config(config),
            edge: BarEdge::from_config(config.and_then(|bar| bar.edge.as_deref())),
        }
    }
}

#[derive(Debug)]
pub enum BarMsg {
    LayoutChanged { layout: BarLayout, edge: BarEdge },
    WorkspacesChanged(Vec<WorkspaceSummary>),
    BatteryChanged(String),
    BatteryUnavailable,
    ClockChanged(String),
    NiriUnavailable(String),
    UpdatesStopped,
}

impl Bar {
    fn configure_window(root: &gtk::ApplicationWindow, edge: BarEdge, name: Option<&str>) {
        root.init_layer_shell();
        root.set_layer(Layer::Top);

        root.set_anchor(Edge::Top, false);
        root.set_anchor(Edge::Bottom, false);
        root.set_anchor(Edge::Left, false);
        root.set_anchor(Edge::Right, false);
        match edge {
            BarEdge::Top => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Left, true);
                root.set_anchor(Edge::Right, true);
            }
            BarEdge::Bottom => {
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Left, true);
                root.set_anchor(Edge::Right, true);
            }
            BarEdge::Left => {
                root.set_anchor(Edge::Left, true);
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Bottom, true);
            }
            BarEdge::Right => {
                root.set_anchor(Edge::Right, true);
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Bottom, true);
            }
        }

        if edge.is_vertical() {
            root.set_size_request(32, -1);
        } else {
            root.set_size_request(-1, 32);
        }

        root.auto_exclusive_zone_enable();
        root.set_keyboard_mode(KeyboardMode::None);
        root.set_namespace(Some(name.unwrap_or("wayward")));
    }

    fn start_missing_watchers(&mut self, sender: &ComponentSender<Self>) {
        for item in self.layout.unique_items() {
            let already_active = self
                .active_watchers
                .iter()
                .any(|(active_item, _)| *active_item == item);

            if already_active {
                continue;
            }

            let handle = registry::start_item(item, sender);
            self.active_watchers.push((item, handle));
        }
    }

    fn stop_removed_watchers(&mut self) {
        let active_layout_items = self.layout.unique_items();

        let mut index = 0;

        while index < self.active_watchers.len() {
            let (item, _) = &self.active_watchers[index];

            if active_layout_items.contains(item) {
                index += 1;
                continue;
            }

            let (item, handle) = self.active_watchers.remove(index);
            handle.abort();
            tracing::info!("Stopped watcher for {item:?}");
        }
    }

    fn reconcile_watchers(&mut self, sender: &ComponentSender<Self>) {
        self.stop_removed_watchers();
        self.start_missing_watchers(sender);
    }

    fn start_config_hot_reload(sender: &ComponentSender<Self>) {
        let Some(dir) = crate::config::config_dir() else {
            tracing::info!("Could not determine config directory, config hot reload disabled");
            return;
        };

        let Some(path) = crate::config::config_path() else {
            tracing::info!("Could not determine config path, config hot reload disabled");
            return;
        };

        let input_sender = sender.input_sender().clone();

        crate::file_watch::start_debounced_file_watch("config", dir, path, move || {
            let config = AppConfig::load();
            let init = BarInit::from_config(config.first_bar());

            if input_sender
                .send(BarMsg::LayoutChanged {
                    layout: init.layout,
                    edge: init.edge,
                })
                .is_err()
            {
                return;
            }

            tracing::info!("Reloaded config");
        });
    }

    fn initial_model(init: BarInit) -> Self {
        Self {
            name: init.name,
            layout: init.layout,
            edge: init.edge,
            active_watchers: Vec::new(),
            workspaces: Vec::new(),
            status: Some("Connecting to Niri".to_string()),
            clock_text: registry::initial_clock_text(),
            battery_text: registry::initial_battery_text(),
        }
    }

    fn render_layout(&self, start_items: &gtk::Box, center_items: &gtk::Box, end_items: &gtk::Box) {
        self.render_region(&self.layout.start, start_items);
        self.render_region(&self.layout.center, center_items);
        self.render_region(&self.layout.end, end_items);
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
impl Component for Bar {
    type Init = BarInit;
    type Input = BarMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Wayward"),
            set_default_height: 32,
            set_resizable: true,

            gtk::CenterBox {
                #[watch]
                set_orientation: model.edge.orientation(),
                #[wrap(Some)]
                #[name = "start_region"]
                set_start_widget = &gtk::Box {
                    add_css_class: "bar-region",

                    #[watch]
                    set_orientation: model.edge.orientation(),
                    #[name = "start_items"]
                    gtk::Box {
                        #[watch]
                        set_orientation: model.edge.orientation(),
                        set_spacing: 4,
                    }
                },

                #[wrap(Some)]
                #[name = "center_region"]
                set_center_widget = &gtk::Box {
                    #[watch]
                    set_hexpand: model.edge.center_hexpand(),
                    #[watch]
                    set_vexpand: model.edge.center_vexpand(),
                    #[watch]
                    set_halign: model.edge.center_halign(),
                    #[watch]
                    set_valign: model.edge.center_valign(),
                    add_css_class: "bar-region",

                    #[watch]
                    set_orientation: model.edge.orientation(),
                    #[name = "center_items"]
                    gtk::Box {
                        #[watch]
                        set_orientation: model.edge.orientation(),
                        set_spacing: 4,
                    }
                },

                #[wrap(Some)]
                #[name = "end_region"]
                set_end_widget = &gtk::Box {
                    add_css_class: "bar-region",

                    #[watch]
                    set_orientation: model.edge.orientation(),
                    #[name = "end_items"]
                    gtk::Box {
                        #[watch]
                        set_orientation: model.edge.orientation(),
                        set_spacing: 4,
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = AppConfig::load();
        if let Some(name) = config.first_bar().and_then(|bar| bar.name.as_deref()) {
            tracing::info!("Starting bar {name}");
        };
        let mut model = Self::initial_model(init);

        if let Some(name) = &model.name {
            tracing::info!("Starting bar {name}");
        }

        Self::configure_window(&root, model.edge, model.name.as_deref());

        let widgets = view_output!();

        model.render_layout(
            &widgets.start_items,
            &widgets.center_items,
            &widgets.end_items,
        );

        model.start_missing_watchers(&sender);
        Self::start_config_hot_reload(&sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            BarMsg::LayoutChanged { layout, edge } => {
                self.layout = layout;
                self.edge = edge;
                Self::configure_window(root, self.edge, self.name.as_deref());
                self.reconcile_watchers(&sender);
            }
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
        self.render_layout(&start_items, &center_items, &end_items);
    }
}
