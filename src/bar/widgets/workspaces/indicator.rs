use relm4::gtk;
use relm4::gtk::glib::ControlFlow;
use relm4::gtk::prelude::{FixedExt, WidgetExt, WidgetExtManual};
use std::{cell::RefCell, rc::Rc};

use super::config::{WorkspaceIndicatorEffect, WorkspacesConfig};

const INDICATOR_HORIZONTAL_OUTSET: f64 = 4.0;
const INDICATOR_VERTICAL_OUTSET: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct IndicatorBounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl IndicatorBounds {
    pub(super) fn from_widget(widget: &gtk::Box, indicator_layer: &gtk::Fixed) -> Option<Self> {
        let bounds = widget.compute_bounds(indicator_layer)?;
        let surface_x = 0.0;
        let surface_end = indicator_layer.width() as f64;
        let is_first = widget.prev_sibling().is_none();
        let is_last = widget.next_sibling().is_none();

        Some(Self {
            x: bounds.x() as f64,
            y: bounds.y() as f64,
            width: bounds.width() as f64,
            height: bounds.height() as f64,
        })
        .map(|bounds| bounds.with_outset(INDICATOR_HORIZONTAL_OUTSET, INDICATOR_VERTICAL_OUTSET))
        .map(|indicator_bounds| {
            indicator_bounds.extend_to_outer_edges(is_first, is_last, surface_x, surface_end)
        })
        .map(|bounds| bounds.clamp_x(surface_x, surface_end))
    }

    pub(super) fn apply_to(self, indicator_layer: &gtk::Fixed, indicator: &gtk::Box) {
        indicator.set_size_request(
            self.width.round().max(0.0) as i32,
            self.height.round().max(0.0) as i32,
        );
        indicator_layer.move_(indicator, self.x, self.y);
        indicator.set_visible(true);
    }

    fn with_outset(self, x_outset: f64, y_outset: f64) -> Self {
        Self {
            x: self.x - x_outset,
            y: self.y - y_outset,
            width: self.width + x_outset * 2.0,
            height: self.height + y_outset * 2.0,
        }
    }

    fn clamp_x(self, min_x: f64, max_x: f64) -> Self {
        if max_x <= min_x {
            return Self {
                x: min_x,
                width: 0.0,
                ..self
            };
        }

        let x = self.x.max(min_x);
        let end = (self.x + self.width).min(max_x);

        Self {
            x,
            width: (end - x).max(0.0),
            ..self
        }
    }

    fn extend_to_outer_edges(
        self,
        is_first: bool,
        is_last: bool,
        outer_x: f64,
        outer_end: f64,
    ) -> Self {
        let mut x = self.x;
        let mut end = self.x + self.width;

        if is_first {
            x = outer_x;
        }

        if is_last {
            end = outer_end;
        }

        Self {
            x,
            width: (end - x).max(0.0),
            ..self
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

    #[test]
    fn bounds_outset_can_preserve_height() {
        let bounds = IndicatorBounds {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };

        assert_eq!(
            bounds.with_outset(4.0, 0.0),
            IndicatorBounds {
                x: 6.0,
                y: 20.0,
                width: 38.0,
                height: 40.0,
            }
        );
    }

    #[test]
    fn bounds_clamp_x_keeps_indicator_within_container() {
        let bounds = IndicatorBounds {
            x: -4.0,
            y: 20.0,
            width: 20.0,
            height: 12.0,
        };

        assert_eq!(
            bounds.clamp_x(0.0, 12.0),
            IndicatorBounds {
                x: 0.0,
                y: 20.0,
                width: 12.0,
                height: 12.0,
            }
        );
    }

    #[test]
    fn bounds_extend_to_outer_start_when_item_touches_inner_start() {
        let bounds = IndicatorBounds {
            x: 6.0,
            y: 20.0,
            width: 20.0,
            height: 12.0,
        };

        assert_eq!(
            bounds.extend_to_outer_edges(true, false, 0.0, 90.0),
            IndicatorBounds {
                x: 0.0,
                y: 20.0,
                width: 26.0,
                height: 12.0,
            }
        );
    }

    #[test]
    fn bounds_extend_to_outer_end_when_item_touches_inner_end() {
        let bounds = IndicatorBounds {
            x: 56.0,
            y: 20.0,
            width: 28.0,
            height: 12.0,
        };

        assert_eq!(
            bounds.extend_to_outer_edges(false, true, 0.0, 90.0),
            IndicatorBounds {
                x: 56.0,
                y: 20.0,
                width: 34.0,
                height: 12.0,
            }
        );
    }

    #[test]
    fn bounds_keep_middle_items_at_outset_size() {
        let bounds = IndicatorBounds {
            x: 26.0,
            y: 20.0,
            width: 28.0,
            height: 12.0,
        };

        assert_eq!(
            bounds.extend_to_outer_edges(false, false, 0.0, 90.0),
            bounds
        );
    }

    #[test]
    fn bounds_extend_to_both_edges_when_only_item() {
        let bounds = IndicatorBounds {
            x: 6.0,
            y: 20.0,
            width: 28.0,
            height: 12.0,
        };

        assert_eq!(
            bounds.extend_to_outer_edges(true, true, 0.0, 90.0),
            IndicatorBounds {
                x: 0.0,
                y: 20.0,
                width: 90.0,
                height: 12.0,
            }
        );
    }
}
