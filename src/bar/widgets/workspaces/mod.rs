mod config;
mod indicator;
pub(crate) mod model;
mod render;
pub(crate) mod service;

use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{BoxExt, FixedExt, WidgetExt};
use std::{cell::RefCell, rc::Rc};

use crate::bar::BarMsg;
use crate::bar::state::{BarItemState, WorkspaceState};
use crate::bar::widget::{BarContext, BarWidget, BarWidgetRuntime, WidgetInstance};
use crate::services::ShellServices;
use crate::shell::ShellMsg;

use self::indicator::{IndicatorAnimationState, IndicatorBounds, start_indicator_animation};
use self::render::{RenderedWorkspace, render_status, render_workspace_state};

pub(crate) const ID: &str = "workspaces";

struct WorkspacesRuntime {
    root: gtk::Box,
    sender: relm4::Sender<BarMsg>,
    indicator_animation: Rc<RefCell<IndicatorAnimationState>>,
    indicator_layer: gtk::Fixed,
    indicator: gtk::Box,
    content: gtk::Box,
    rendered_workspaces: Vec<RenderedWorkspace>,
    config: config::WorkspacesConfig,
}

impl BarWidgetRuntime for WorkspacesRuntime {
    fn root(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        let BarItemState::Workspaces(state) = state else {
            return;
        };

        let active_workspace = render_workspace_state(
            &self.content,
            &mut self.rendered_workspaces,
            state,
            context.monitor_connector.as_deref(),
            &self.config.label_format,
            &self.sender,
        );

        self.update_indicator(active_workspace);
    }
}

impl WorkspacesRuntime {
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

pub(crate) struct WorkspacesWidget;

impl BarWidget for WorkspacesWidget {
    fn id(&self) -> &'static str {
        ID
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        sender: &relm4::Sender<BarMsg>,
        _services: &crate::services::ShellServices,
        context: &BarContext,
    ) -> Box<dyn BarWidgetRuntime> {
        let config = instance.config_as::<config::WorkspacesConfig>();
        let orientation = context.edge.orientation();

        let root = gtk::Box::new(orientation, 0);
        let instance_class = instance.instance_css_class();
        crate::bar::style::add_bar_item_classes(&root, "workspaces", instance_class.as_deref());

        let overlay = gtk::Overlay::new();
        overlay.add_css_class("workspaces-overlay");
        root.append(&overlay);

        let base = gtk::Box::new(orientation, 0);
        base.add_css_class("workspaces-base");
        overlay.set_child(Some(&base));

        let indicator_layer = gtk::Fixed::new();
        indicator_layer.add_css_class("workspaces-indicator-layer");
        overlay.add_overlay(&indicator_layer);
        overlay.set_measure_overlay(&indicator_layer, false);

        let indicator = gtk::Box::new(orientation, 0);
        indicator.add_css_class("workspace-indicator");
        indicator.set_can_target(false);
        indicator.set_visible(false);
        indicator_layer.put(&indicator, 0.0, 0.0);

        let content = gtk::Box::new(orientation, 4);
        content.add_css_class("bar-item-content");
        content.add_css_class("workspaces-content");
        overlay.add_overlay(&content);
        overlay.set_measure_overlay(&content, true);

        render_status(&content, "Connecting to Niri");

        Box::new(WorkspacesRuntime {
            root,
            sender: sender.clone(),
            indicator_animation: Rc::new(RefCell::new(IndicatorAnimationState::default())),
            indicator_layer,
            indicator,
            content,
            rendered_workspaces: Vec::new(),
            config,
        })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Workspaces(WorkspaceState::Connecting))
    }

    fn start(
        &self,
        sender: relm4::Sender<ShellMsg>,
        services: &ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        Some(service::start_workspace_watcher(
            sender,
            services.niri.clone(),
        ))
    }
}
