mod config;
pub(crate) mod model;
mod render;
pub(crate) mod service;

use relm4::gtk;
use relm4::gtk::glib::ControlFlow;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{BoxExt, FixedExt, WidgetExt, WidgetExtManual};
use std::{cell::RefCell, rc::Rc};

use crate::bar::BarMsg;
use crate::bar::state::{BarItemState, WorkspaceState};
use crate::bar::widget::{BarContext, BarWidget, BarWidgetRuntime, WidgetInstance};
use crate::services::ShellServices;
use crate::shell::ShellMsg;

use self::render::{RenderedWorkspace, render_status, render_workspace_state};

#[derive(Clone, Copy, Debug)]
struct IndicatorBounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl IndicatorBounds {
    fn apply_to(self, indicator_layer: &gtk::Fixed, indicator: &gtk::Box) {
        indicator.set_size_request(
            self.width.round().max(0.0) as i32,
            self.height.round().max(0.0) as i32,
        );
        indicator_layer.move_(indicator, self.x, self.y);
        indicator.set_visible(true);
    }

    fn lerp(self, target: Self, progress: f64) -> Self {
        Self {
            x: lerp(self.x, target.x, progress),
            y: lerp(self.y, target.y, progress),
            width: lerp(self.width, target.width, progress),
            height: lerp(self.height, target.height, progress),
        }
    }
}

fn lerp(start: f64, end: f64, progress: f64) -> f64 {
    start + (end - start) * progress
}

fn animation_progress(effect: config::WorkspaceIndicatorEffect, progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);

    match effect {
        config::WorkspaceIndicatorEffect::None => 1.0,
        config::WorkspaceIndicatorEffect::Slide => progress,
        config::WorkspaceIndicatorEffect::Ease => {
            let inverse = 1.0 - progress;
            1.0 - inverse * inverse * inverse
        }
    }
}

#[derive(Default)]
struct IndicatorAnimationState {
    current: Option<IndicatorBounds>,
    callback: Option<gtk::TickCallbackId>,
}

struct WorkspacesRuntime {
    root: gtk::Box,
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
            let Some(bounds) = active_workspace.compute_bounds(&indicator_layer) else {
                let mut state = animation_state.borrow_mut();

                if let Some(callback) = state.callback.take() {
                    callback.remove();
                }

                state.current = None;
                indicator.set_visible(false);
                return;
            };

            let target = IndicatorBounds {
                x: bounds.x() as f64,
                y: bounds.y() as f64,
                width: bounds.width() as f64,
                height: bounds.height() as f64,
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

fn start_indicator_animation(
    indicator_layer: gtk::Fixed,
    indicator: gtk::Box,
    animation_state: Rc<RefCell<IndicatorAnimationState>>,
    config: config::WorkspacesConfig,
    start: IndicatorBounds,
    target: IndicatorBounds,
) {
    let tick_widget = indicator.clone();
    let callback_indicator = indicator.clone();
    let callback_layer = indicator_layer.clone();
    let callback_state = animation_state.clone();
    let started_at = Rc::new(RefCell::new(None::<i64>));
    let callback_started_at = started_at.clone();

    let callback_id = tick_widget.add_tick_callback(move |_, frame_clock| {
        let start_time = {
            let mut started_at = callback_started_at.borrow_mut();
            *started_at.get_or_insert_with(|| frame_clock.frame_time())
        };

        let elapsed_ms = (frame_clock.frame_time() - start_time) as f64 / 1000.0;
        let raw_progress = (elapsed_ms / config.indicator_duration_ms as f64).clamp(0.0, 1.0);
        let progress = animation_progress(config.indicator_effect, raw_progress);

        let frame_bounds = start.lerp(target, progress);
        frame_bounds.apply_to(&callback_layer, &callback_indicator);

        let mut state = callback_state.borrow_mut();
        state.current = Some(frame_bounds);

        if raw_progress >= 1.0 {
            state.current = Some(target);
            state.callback = None;
            ControlFlow::Break
        } else {
            ControlFlow::Continue
        }
    });

    animation_state.borrow_mut().callback = Some(callback_id);
}

pub(crate) struct WorkspacesWidget;

impl BarWidget for WorkspacesWidget {
    fn id(&self) -> &'static str {
        "workspaces"
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        _sender: &relm4::Sender<BarMsg>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slide_progress_is_linear() {
        assert_eq!(
            animation_progress(config::WorkspaceIndicatorEffect::Slide, 0.5),
            0.5
        );
    }

    #[test]
    fn ease_progress_moves_faster_than_linear_at_halfway() {
        assert_eq!(
            animation_progress(config::WorkspaceIndicatorEffect::Ease, 0.5),
            0.875
        );
    }

    #[test]
    fn none_progress_finishes_immediately() {
        assert_eq!(
            animation_progress(config::WorkspaceIndicatorEffect::None, 0.25),
            1.0
        );
    }
}
