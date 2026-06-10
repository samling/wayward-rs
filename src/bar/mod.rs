mod style;
mod window;

pub(crate) mod dropdown;
pub(crate) mod layout;
pub(crate) mod registry;
pub(crate) mod state;
pub(crate) mod widget;
pub(crate) mod widgets;

use crate::shell::ShellMsg;
use layout::{BarEdge, BarLayout};
use state::BarItemState;
use widget::{
    BarContext, BarRegion, BarWidgetRuntime, WidgetAction, WidgetBuildContext, WidgetEvent,
    WidgetInstance,
};

use gtk::prelude::*;
use relm4::gtk;
use relm4::prelude::*;

pub struct BarInit {
    pub(crate) name: Option<String>,
    pub(crate) layout: BarLayout,
    pub(crate) edge: BarEdge,
    pub(crate) monitor: Option<gtk::gdk::Monitor>,
    pub(crate) monitor_connector: Option<String>,
    pub(crate) services: crate::services::ShellServices,
}

impl BarInit {
    pub(crate) fn from_config(
        app_config: &crate::config::AppConfig,
        config: Option<&crate::config::BarConfig>,
        monitor: Option<gtk::gdk::Monitor>,
        services: crate::services::ShellServices,
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
            services,
        }
    }
}

#[derive(Debug)]
pub enum BarMsg {
    LayoutChanged { layout: BarLayout, edge: BarEdge },
    ItemStateChanged(BarItemState),
    WidgetEvent(WidgetEvent),
    StyleChanged,
}

struct MountedWidget {
    instance: WidgetInstance,
    widget_id: &'static str,
    region: BarRegion,
    runtime: Box<dyn BarWidgetRuntime>,
}

#[derive(Default)]
struct MountedLayout {
    start: Vec<MountedWidget>,
    center: Vec<MountedWidget>,
    end: Vec<MountedWidget>,
}

impl MountedWidget {
    fn root(&self) -> gtk::Widget {
        self.runtime.root()
    }

    fn matches_instance(&self, instance: &WidgetInstance) -> bool {
        self.instance == *instance
    }
}

pub(crate) struct Bar {
    name: Option<String>,
    layout: BarLayout,
    mounted_layout: MountedLayout,
    edge: BarEdge,
    monitor: Option<gtk::gdk::Monitor>,
    monitor_connector: Option<String>,
    input_sender: relm4::Sender<BarMsg>,
    pub(super) item_states: Vec<BarItemState>,
    services: crate::services::ShellServices,
}

impl Bar {
    fn context(&self, region: BarRegion) -> BarContext {
        BarContext {
            monitor_connector: self.monitor_connector.clone(),
            edge: self.edge,
            region,
        }
    }

    fn mounted_widgets_mut(&mut self) -> impl Iterator<Item = &mut MountedWidget> {
        self.mounted_layout
            .start
            .iter_mut()
            .chain(self.mounted_layout.center.iter_mut())
            .chain(self.mounted_layout.end.iter_mut())
    }

    fn apply_context_to_mounted_widgets(&mut self) {
        let edge = self.edge;
        let monitor_connector = self.monitor_connector.clone();

        for mounted in self.mounted_widgets_mut() {
            let context = BarContext {
                monitor_connector: monitor_connector.clone(),
                edge,
                region: mounted.region,
            };

            mounted.runtime.set_context(&context);
        }
    }

    fn apply_state_to_mounted_widgets(&mut self, state: &BarItemState) {
        let widget_id = state.widget_id();
        let edge = self.edge;
        let monitor_connector = self.monitor_connector.clone();

        for mounted in self.mounted_widgets_mut() {
            if mounted.widget_id == widget_id {
                let context = BarContext {
                    monitor_connector: monitor_connector.clone(),
                    edge,
                    region: mounted.region,
                };
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
            services: init.services,
        }
    }

    fn mount_layout(
        &mut self,
        start_items: &gtk::Box,
        center_items: &gtk::Box,
        end_items: &gtk::Box,
    ) {
        self.mounted_layout.start =
            self.mount_region(BarRegion::Start, &self.layout.start, start_items);
        self.mounted_layout.center =
            self.mount_region(BarRegion::Center, &self.layout.center, center_items);
        self.mounted_layout.end = self.mount_region(BarRegion::End, &self.layout.end, end_items);
    }

    fn mount_widget(
        &self,
        region: BarRegion,
        instance: &WidgetInstance,
        container: &gtk::Box,
    ) -> MountedWidget {
        let context = self.context(region);
        let build_context = WidgetBuildContext {
            sender: &self.input_sender,
            services: &self.services,
            bar: &context,
        };

        let runtime = instance.widget.build(instance, &build_context);
        let root = runtime.root();

        container.append(&root);

        MountedWidget {
            instance: instance.clone(),
            widget_id: instance.widget.id(),
            region,
            runtime,
        }
    }

    fn mount_region(
        &self,
        region: BarRegion,
        widgets: &[WidgetInstance],
        container: &gtk::Box,
    ) -> Vec<MountedWidget> {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        widgets
            .iter()
            .map(|instance| self.mount_widget(region, instance, container))
            .collect()
    }

    fn reconcile_layout(
        &mut self,
        start_items: &gtk::Box,
        center_items: &gtk::Box,
        end_items: &gtk::Box,
    ) {
        let start_widgets = self.layout.start.clone();
        let start_mounted = std::mem::take(&mut self.mounted_layout.start);
        let start =
            self.reconcile_region(BarRegion::Start, &start_widgets, start_mounted, start_items);
        self.mounted_layout.start = start;

        let center_widgets = self.layout.center.clone();
        let center_mounted = std::mem::take(&mut self.mounted_layout.center);
        let center = self.reconcile_region(
            BarRegion::Center,
            &center_widgets,
            center_mounted,
            center_items,
        );
        self.mounted_layout.center = center;

        let end_widgets = self.layout.end.clone();
        let end_mounted = std::mem::take(&mut self.mounted_layout.end);
        let end = self.reconcile_region(BarRegion::End, &end_widgets, end_mounted, end_items);
        self.mounted_layout.end = end;
    }

    fn reconcile_region(
        &self,
        region: BarRegion,
        widgets: &[WidgetInstance],
        mut mounted: Vec<MountedWidget>,
        container: &gtk::Box,
    ) -> Vec<MountedWidget> {
        let mut reconciled: Vec<MountedWidget> = Vec::new();

        for instance in widgets {
            if let Some(index) = mounted
                .iter()
                .position(|mounted| mounted.matches_instance(instance))
            {
                let mounted = mounted.remove(index);
                let root = mounted.root();
                if let Some(previous) = reconciled.last() {
                    container.reorder_child_after(&root, Some(&previous.root()));
                } else {
                    container.reorder_child_after(&root, None::<&gtk::Widget>);
                }

                reconciled.push(mounted);
            } else {
                reconciled.push(self.mount_widget(region, instance, container));
            }
        }

        for mounted in mounted {
            container.remove(&mounted.root());
        }

        reconciled
    }
}

#[relm4::component(pub(crate))]
impl Component for Bar {
    type Init = BarInit;
    type Input = BarMsg;
    type Output = ShellMsg;
    type CommandOutput = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Wayward"),
            set_default_height: 1,
            set_resizable: true,
            add_css_class: "bar",

            gtk::CenterBox {
                #[watch]
                set_orientation: model.edge.orientation(),
                #[wrap(Some)]
                #[name = "start_region"]
                set_start_widget = &gtk::Box {
                    #[watch]
                    set_visible: !model.layout.start.is_empty(),
                    add_css_class: "bar-region",
                    add_css_class: "bar-region-start",

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
                    #[watch]
                    set_visible: !model.layout.center.is_empty(),
                    add_css_class: "bar-region",
                    add_css_class: "bar-region-center",

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
                    #[watch]
                    set_visible: !model.layout.end.is_empty(),
                    add_css_class: "bar-region",
                    add_css_class: "bar-region-end",

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

        window::configure_window(
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
                let edge_changed = self.edge != edge;

                self.layout = layout;
                self.edge = edge;

                if edge_changed {
                    window::configure_window(
                        root,
                        self.edge,
                        self.name.as_deref(),
                        self.monitor.as_ref(),
                    );
                }

                self.reconcile_layout(
                    &widgets.start_items,
                    &widgets.center_items,
                    &widgets.end_items,
                );

                self.apply_context_to_mounted_widgets();
                self.apply_all_states_to_mounted_widgets();
            }
            BarMsg::StyleChanged => {
                window::apply_size_hint(root, self.edge);
                root.queue_resize();
                root.queue_draw();
            }
            BarMsg::ItemStateChanged(state) => {
                self.item_states
                    .retain(|existing_state| !existing_state.same_widget_as(&state));

                self.item_states.push(state.clone());
                self.apply_state_to_mounted_widgets(&state);
            }
            BarMsg::WidgetEvent(event) => {
                if matches!(event.action, WidgetAction::OpenSettings) {
                    let _ = sender.output(ShellMsg::OpenSettings);
                } else {
                    registry::handle_widget_event(event, &self.services);
                }
            }
        }

        self.update_view(widgets, sender);
    }

    fn pre_view() {}
}
