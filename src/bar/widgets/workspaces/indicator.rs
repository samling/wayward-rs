use relm4::gtk;
use relm4::gtk::glib::ControlFlow;
use relm4::gtk::prelude::{FixedExt, WidgetExt, WidgetExtManual};
use std::{cell::RefCell, rc::Rc};

use crate::bar::layout::BarEdge;

use super::config::{WorkspaceIndicatorEffect, WorkspacesConfig};

const INDICATOR_HORIZONTAL_OUTSET: f64 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct IndicatorBounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl IndicatorBounds {
    pub(super) fn from_widget(widget: &gtk::Box, surface: &impl WidgetExt, edge: BarEdge) -> Option<Self> {
        let bounds = widget.compute_bounds(surface)?;
        let surface_x = 0.0;
        let surface_y = 0.0;
        let surface_width = surface.width() as f64;
        let surface_height = surface.height() as f64;
        let is_first = widget.prev_sibling().is_none();
        let is_last = widget.next_sibling().is_none();

        let bounds = match edge {
            BarEdge::Top | BarEdge::Bottom => Self {
                x: bounds.x() as f64,
                y: surface_y,
                width: bounds.width() as f64,
                height: surface_height,
            },
            BarEdge::Left | BarEdge::Right => Self {
                x: surface_x,
                y: bounds.y() as f64,
                width: surface_width,
                height: bounds.height() as f64,
            },
        };

        Some(bounds)
            .map(|bounds| bounds.with_outset(edge, INDICATOR_HORIZONTAL_OUTSET))
            .map(|indicator_bounds| {
                indicator_bounds.extend_to_outer_edges(
                    edge,
                    is_first,
                    is_last,
                    surface_x,
                    surface_y,
                    surface_width,
                    surface_height,
                )
            })
            .map(|bounds| bounds.clamp(edge, surface_x, surface_y, surface_width, surface_height))
    }

    pub(super) fn apply_to(self, indicator_layer: &gtk::Fixed, indicator: &gtk::Box) {
        indicator.set_size_request(
            self.width.round().max(0.0) as i32,
            self.height.round().max(0.0) as i32,
        );
        indicator_layer.move_(indicator, self.x, self.y);
        indicator.set_visible(true);
    }

    fn with_outset(self, edge: BarEdge, outset: f64) -> Self {
        match edge {
        BarEdge::Top | BarEdge::Bottom => Self {
            x: self.x - outset,
            width: self.width + outset * 2.0,
            ..self
        },
        BarEdge::Left | BarEdge::Right => Self {
            y: self.y - outset,
            height: self.height + outset * 2.0,
            ..self
            },
        }
    }

    fn clamp(
        self,
        edge: BarEdge,
        min_x: f64,
        min_y: f64,
        surface_width: f64,
        surface_height: f64,
    ) -> Self {
        match edge {
            BarEdge::Top | BarEdge::Bottom => self.clamp_main_axis(min_x, surface_width),
            BarEdge::Left | BarEdge::Right => self.clamp_cross_axis(min_y, surface_height),
        }
    }

    fn clamp_main_axis(self, min_x: f64, surface_width: f64) -> Self {
        if surface_width <= min_x {
            return Self {
                x: min_x,
                width: 0.0,
                ..self
            };
        }

        let x = self.x.max(min_x);
        let end = (self.x + self.width).min(surface_width);

        Self {
            x,
            width: (end - x).max(0.0),
            ..self
        }
    }

    fn clamp_cross_axis(self, min_y: f64, surface_height: f64) -> Self {
        if surface_height <= min_y {
            return Self {
                y: min_y,
                height: 0.0,
                ..self
            };
        }

        let y = self.y.max(min_y);
        let end = (self.y + self.height).min(surface_height);

        Self {
            y,
            height: (end - y).max(0.0),
            ..self
        }
    }

    fn extend_to_outer_edges(
        self,
        edge: BarEdge,
        is_first: bool,
        is_last: bool,
        outer_x: f64,
        outer_y: f64,
        surface_width: f64,
        surface_height: f64,
    ) -> Self {
        match edge {
            BarEdge::Top | BarEdge::Bottom => {
                let mut x = self.x;
                let mut end = self.x + self.width;

                if is_first {
                    x = outer_x;
                }

                if is_last {
                    end = surface_width;
                }

                Self {
                    x,
                    width: (end - x).max(0.0),
                    ..self
                }
            }
            BarEdge::Left | BarEdge::Right => {
                let mut y = self.y;
                let mut end = self.y + self.height;

                if is_first {
                    y = outer_y;
                }

                if is_last {
                    end = surface_height;
                }

                Self {
                    y,
                    height: (end - y).max(0.0),
                    ..self
                }
            }
        }
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

fn animation_progress(effect: WorkspaceIndicatorEffect, progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);

    match effect {
        WorkspaceIndicatorEffect::None => 1.0,
        WorkspaceIndicatorEffect::Slide => progress,
        WorkspaceIndicatorEffect::Ease => {
            let inverse = 1.0 - progress;
            1.0 - inverse * inverse * inverse
        }
    }
}

#[derive(Default)]
pub(super) struct IndicatorAnimationState {
    pub(super) current: Option<IndicatorBounds>,
    pub(super) callback: Option<gtk::TickCallbackId>,
}

pub(super) fn start_indicator_animation(
    indicator_layer: gtk::Fixed,
    indicator: gtk::Box,
    animation_state: Rc<RefCell<IndicatorAnimationState>>,
    config: WorkspacesConfig,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slide_progress_is_linear() {
        assert_eq!(
            animation_progress(WorkspaceIndicatorEffect::Slide, 0.5),
            0.5
        );
    }

    #[test]
    fn ease_progress_moves_faster_than_linear_at_halfway() {
        assert_eq!(
            animation_progress(WorkspaceIndicatorEffect::Ease, 0.5),
            0.875
        );
    }

    #[test]
    fn none_progress_finishes_immediately() {
        assert_eq!(
            animation_progress(WorkspaceIndicatorEffect::None, 0.25),
            1.0
        );
    }
}
