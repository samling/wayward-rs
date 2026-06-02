mod style;

pub(crate) mod dropdown;
pub(crate) mod layout;
pub(crate) mod registry;
pub(crate) mod state;
pub(crate) mod widget;
pub(crate) mod widgets;

use layout::{BarEdge, BarLayout};
use state::BarItemState;
use widget::{BarContext, BarWidgetRuntime, WidgetEvent, WidgetInstance};

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;
use relm4::prelude::*;

pub struct BarInit {
    pub(crate) name: Option<String>,
    pub(crate) layout: BarLayout,
    pub(crate) edge: BarEdge,
    pub(crate) monitor: Option<gtk::gdk::Monitor>,
    pub(crate) monitor_connector: Option<String>,
}

impl BarInit {
    pub(crate) fn from_config(
        app_config: &crate::config::AppConfig,
        config: Option<&crate::config::BarConfig>,
        monitor: Option<gtk::gdk::Monitor>,
    ) -> Self {
        let monitor_connector = monitor
            .as_ref()
            .and_then(|monitor| monitor.connector().map(|connector| connector.to_string()));
        Self {
            name: config.and_then(|bar| bar.name.clone()),
            layout: BarLayout::from_config(app_config, config),
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
    WidgetEvent(WidgetEvent),
}

#[derive(Debug)]
pub enum BarOutput {
    WidgetEvent(WidgetEvent),
}

struct MountedWidget {
    widget_id: &'static str,
    runtime: Box<dyn BarWidgetRuntime>,
}

#[derive(Default)]
struct MountedLayout {
    start: Vec<MountedWidget>,
    center: Vec<MountedWidget>,
    end: Vec<MountedWidget>,
}

pub struct Bar {
    name: Option<String>,
    layout: BarLayout,
    mounted_layout: MountedLayout,
    edge: BarEdge,
    monitor: Option<gtk::gdk::Monitor>,
    monitor_connector: Option<String>,
    input_sender: relm4::Sender<BarMsg>,
    pub(super) item_states: Vec<BarItemState>,
}

impl Bar {
    fn context(&self) -> BarContext {
        BarContext {
            monitor_connector: self.monitor_connector.clone(),
            edge: self.edge,
        }
    }

    fn mounted_widgets_mut(&mut self) -> impl Iterator<Item = &mut MountedWidget> {
        self.mounted_layout
            .start
            .iter_mut()
            .chain(self.mounted_layout.center.iter_mut())
            .chain(self.mounted_layout.end.iter_mut())
    }

    fn apply_state_to_mounted_widgets(&mut self, state: &BarItemState) {
        let context = self.context();
        let widget_id = state.widget_id();

        for mounted in self.mounted_widgets_mut() {
            if mounted.widget_id == widget_id {
                mounted.runtime.update(state, &context);
            }
        }
    }

    fn apply_all_states_to_mounted_widgets(&mut self) {
        let states = self.item_states.clone();

        for state in states {
            self.apply_state_to_mounted_widgets(&state);
        }
    }

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

    fn initial_model(init: BarInit, input_sender: relm4::Sender<BarMsg>) -> Self {
        Self {
            name: init.name,
            layout: init.layout,
            mounted_layout: MountedLayout::default(),
            edge: init.edge,
            monitor: init.monitor,
            monitor_connector: init.monitor_connector,
            input_sender,
            item_states: registry::WIDGETS
                .iter()
                .filter_map(|widget| widget.initial_state())
                .collect(),
        }
    }

    fn mount_layout(
        &mut self,
        start_items: &gtk::Box,
        center_items: &gtk::Box,
        end_items: &gtk::Box,
    ) {
        self.mounted_layout.start = self.mount_region(&self.layout.start, start_items);
        self.mounted_layout.center = self.mount_region(&self.layout.center, center_items);
        self.mounted_layout.end = self.mount_region(&self.layout.end, end_items);
    }

    fn mount_region(&self, widgets: &[WidgetInstance], container: &gtk::Box) -> Vec<MountedWidget> {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        widgets
            .iter()
            .map(|instance| {
                let runtime = instance.widget.build(instance, &self.input_sender.clone());
                let root = runtime.root();

                container.append(&root);

                MountedWidget {
                    widget_id: instance.widget.id(),
                    runtime,
                }
            })
            .collect()
    }
}

#[relm4::component(pub)]
impl Component for Bar {
    type Init = BarInit;
    type Input = BarMsg;
    type Output = BarOutput;
    type CommandOutput = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Wayward"),
            set_default_height: 32,
            set_resizable: true,
            add_css_class: "bar",

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
        let mut model = Self::initial_model(init, sender.input_sender().clone());

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

        model.mount_layout(
            &widgets.start_items,
            &widgets.center_items,
            &widgets.end_items,
        );

        root.present();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
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

                self.mount_layout(
                    &widgets.start_items,
                    &widgets.center_items,
                    &widgets.end_items,
                );

                self.apply_all_states_to_mounted_widgets();
            }
            BarMsg::ItemStateChanged(state) => {
                self.item_states
                    .retain(|existing_state| !existing_state.same_widget_as(&state));

                self.item_states.push(state.clone());
                self.apply_state_to_mounted_widgets(&state);
            }
            BarMsg::WidgetEvent(event) => {
                let _ = sender.output(BarOutput::WidgetEvent(event));
            }
        }

        self.update_view(widgets, sender);
    }

    fn pre_view() {}
}
