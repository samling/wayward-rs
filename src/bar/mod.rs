mod style;
mod widget;

pub(crate) mod layout;
pub(crate) mod registry;
pub(crate) mod state;
pub(crate) mod widgets;

use layout::{BarEdge, BarLayout};
use state::BarItemState;
use widget::BarWidget;

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;
use relm4::prelude::*;

pub struct Bar {
    name: Option<String>,
    layout: BarLayout,
    edge: BarEdge,
    monitor: Option<gtk::gdk::Monitor>,
    monitor_connector: Option<String>,
    pub(super) item_states: Vec<BarItemState>,
}

pub struct BarInit {
    pub(crate) name: Option<String>,
    pub(crate) layout: BarLayout,
    pub(crate) edge: BarEdge,
    pub(crate) monitor: Option<gtk::gdk::Monitor>,
    pub(crate) monitor_connector: Option<String>,
}

impl BarInit {
    pub(crate) fn from_config(
        config: Option<&crate::config::BarConfig>,
        monitor: Option<gtk::gdk::Monitor>,
    ) -> Self {
        let monitor_connector = monitor
            .as_ref()
            .and_then(|monitor| monitor.connector().map(|connector| connector.to_string()));
        Self {
            name: config.and_then(|bar| bar.name.clone()),
            layout: BarLayout::from_config(config),
            edge: BarEdge::from_config(config.and_then(|bar| bar.edge.as_deref())),
            monitor,
            monitor_connector,
        }
    }
}

#[derive(Debug)]
pub enum BarMsg {
    LayoutChanged { layout: BarLayout, edge: BarEdge },
    ItemStateChanged(BarItemState),
}

impl Bar {
    fn configure_window(
        root: &gtk::ApplicationWindow,
        edge: BarEdge,
        name: Option<&str>,
        monitor: Option<&gtk::gdk::Monitor>,
    ) {
        root.init_layer_shell();
        root.set_monitor(monitor);
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

    fn initial_model(init: BarInit) -> Self {
        Self {
            name: init.name,
            layout: init.layout,
            edge: init.edge,
            monitor: init.monitor,
            monitor_connector: init.monitor_connector,
            item_states: registry::WIDGETS
                .iter()
                .filter_map(|widget| widget.initial_state())
                .collect(),
        }
    }

    fn render_layout(&self, start_items: &gtk::Box, center_items: &gtk::Box, end_items: &gtk::Box) {
        self.render_region(&self.layout.start, start_items);
        self.render_region(&self.layout.center, center_items);
        self.render_region(&self.layout.end, end_items);
    }

    fn render_region(&self, widgets: &[&'static dyn BarWidget], container: &gtk::Box) {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        for widget in widgets {
            widget.render(self, container);
        }
    }

    pub(super) fn item_states(&self) -> &[BarItemState] {
        &self.item_states
    }

    pub(super) fn monitor_connector(&self) -> Option<&str> {
        self.monitor_connector.as_deref()
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
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self::initial_model(init);

        if let Some(name) = &model.name {
            tracing::info!("Starting bar {name}");
        }

        Self::configure_window(
            &root,
            model.edge,
            model.name.as_deref(),
            model.monitor.as_ref(),
        );

        let widgets = view_output!();

        model.render_layout(
            &widgets.start_items,
            &widgets.center_items,
            &widgets.end_items,
        );

        root.present();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            BarMsg::LayoutChanged { layout, edge } => {
                self.layout = layout;
                self.edge = edge;
                Self::configure_window(
                    root,
                    self.edge,
                    self.name.as_deref(),
                    self.monitor.as_ref(),
                );
            }
            BarMsg::ItemStateChanged(state) => {
                self.item_states
                    .retain(|existing_state| !existing_state.same_widget_as(&state));

                self.item_states.push(state);
            }
        }
    }

    fn pre_view() {
        self.render_layout(&start_items, &center_items, &end_items);
    }
}
