use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use std::{cell::RefCell, rc::Rc};

use crate::bar::BarMsg;
use crate::bar::layout::BarEdge;
use crate::bar::state::WorkspaceState;

use super::config::{self, WorkspacesConfig};
use super::indicator::{IndicatorAnimationState, IndicatorBounds, start_indicator_animation};
use super::render::{RenderedWorkspace, render_status, render_workspace_state};

pub(super) struct WorkspacesComponent {
    sender: relm4::Sender<BarMsg>,
    indicator_animation: Rc<RefCell<IndicatorAnimationState>>,
    indicator_layer: gtk::Fixed,
    indicator: gtk::Box,
    content: gtk::Box,
    rendered_workspaces: Vec<RenderedWorkspace>,
    config: WorkspacesConfig,
    monitor_connector: Option<String>,
    edge: BarEdge,
}

pub(super) struct WorkspacesInit {
    pub(super) edge: BarEdge,
    pub(super) monitor_connector: Option<String>,
    pub(super) config: WorkspacesConfig,
    pub(super) instance_class: Option<String>,
    pub(super) sender: relm4::Sender<BarMsg>,
}

#[derive(Debug)]
pub(super) enum WorkspacesInput {
    SetState {
        state: WorkspaceState,
        monitor_connector: Option<String>,
    },
    SetPlacement {
        edge: BarEdge,
        monitor_connector: Option<String>,
    },
}

pub(super) struct WorkspacesWidgets;

impl SimpleComponent for WorkspacesComponent {
    type Init = WorkspacesInit;
    type Input = WorkspacesInput;
    type Output = ();
    type Root = gtk::Box;
    type Widgets = WorkspacesWidgets;

    fn init_root() -> Self::Root {
        gtk::Box::new(gtk::Orientation::Horizontal, 0)
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let orientation = init.edge.orientation();

        root.set_orientation(orientation);
        crate::bar::style::add_bar_item_classes(&root, super::ID, init.instance_class.as_deref());

        let overlay = gtk::Overlay::new();
        crate::bar::style::add_bar_item_content_classes(&overlay, "workspaces-content");
        root.append(&overlay);

        let indicator_layer = gtk::Fixed::new();
        indicator_layer.add_css_class("workspaces-indicator-layer");
        overlay.set_child(Some(&indicator_layer));

        let indicator = gtk::Box::new(orientation, 0);
        indicator.add_css_class("workspace-indicator");
        indicator.set_can_target(false);
        indicator.set_visible(false);
        indicator_layer.put(&indicator, 0.0, 0.0);

        let content = gtk::Box::new(orientation, 4);
        content.add_css_class("workspaces-items");
        overlay.add_overlay(&content);
        overlay.set_measure_overlay(&content, true);

        render_status(&content, "Connecting to Niri");

        let model = Self {
            sender: init.sender,
            indicator_animation: Rc::new(RefCell::new(IndicatorAnimationState::default())),
            indicator_layer,
            indicator,
            content,
            rendered_workspaces: Vec::new(),
            config: init.config,
            monitor_connector: init.monitor_connector,
            edge: init.edge,
        };

        ComponentParts {
            model,
            widgets: WorkspacesWidgets,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            WorkspacesInput::SetState {
                state,
                monitor_connector,
            } => {
                self.monitor_connector = monitor_connector;

                let active_workspace = render_workspace_state(
                    &self.content,
                    &mut self.rendered_workspaces,
                    &state,
                    self.monitor_connector.as_deref(),
                    &self.config.label_format,
                    &self.sender,
                );

                self.update_indicator(active_workspace);
            }
            WorkspacesInput::SetPlacement {
                edge,
                monitor_connector,
            } => {
                self.edge = edge;
                self.monitor_connector = monitor_connector;

                let orientation = self.edge.orientation();
                self.content.set_orientation(orientation);
                self.indicator.set_orientation(orientation);

                for rendered in &self.rendered_workspaces {
                    rendered.root.set_orientation(orientation);
                }
            }
        }
    }
}

impl WorkspacesComponent {
    fn update_indicator(&self, active_workspace: Option<gtk::Box>) {
        let Some(active_workspace) = active_workspace else {
            let mut state = self.indicator_animation.borrow_mut();

            if let Some(callback) = state.callback.take() {
                callback.remove();
            }

            state.current = None;
            self.indicator.set_visible(false);
            return;
        };

        let indicator_layer = self.indicator_layer.clone();
        let indicator = self.indicator.clone();
        let animation_state = self.indicator_animation.clone();
        let config = self.config.clone();

        gtk::glib::idle_add_local_once(move || {
            let Some(target) = IndicatorBounds::from_widget(&active_workspace, &indicator_layer)
            else {
                let mut state = animation_state.borrow_mut();

                if let Some(callback) = state.callback.take() {
                    callback.remove();
                }

                state.current = None;
                indicator.set_visible(false);
                return;
            };

            let mut state = animation_state.borrow_mut();

            if let Some(callback) = state.callback.take() {
                callback.remove();
            }

            let has_current = state.current.is_some();
            let start = state.current.unwrap_or(target);

            if !has_current
                || config.indicator_effect == config::WorkspaceIndicatorEffect::None
                || config.indicator_duration_ms == 0
            {
                target.apply_to(&indicator_layer, &indicator);
                state.current = Some(target);
                return;
            }

            drop(state);

            start_indicator_animation(
                indicator_layer,
                indicator,
                animation_state,
                config,
                start,
                target,
            );
        });
    }
}
