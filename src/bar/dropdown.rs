use gtk::prelude::*;
use relm4::gtk;
use relm4::gtk::glib;
use std::time::Duration;

use crate::bar::layout::BarEdge;
use crate::bar::widget::BarRegion;

const DROPDOWN_GAP: i32 = 6;
pub(crate) const TRANSITION_MS: u32 = 140;

pub(crate) fn install_revealer(
    popover: &gtk::Popover,
    revealer: &gtk::Revealer,
    edge: BarEdge,
    region: BarRegion,
) {
    revealer.set_transition_duration(TRANSITION_MS);
    revealer.set_reveal_child(false);

    if let Some(child) = popover.child() {
        popover.set_child(None::<&gtk::Widget>);
        revealer.set_child(Some(&child));
    }

    popover.set_child(Some(revealer));
    set_placement(popover, revealer, edge, region);

    connect_revealer(popover, revealer);
}

pub(crate) fn connect_revealer(popover: &gtk::Popover, revealer: &gtk::Revealer) {
    let revealer_on_map = revealer.clone();
    popover.connect_map(move |_| {
        let revealer = revealer_on_map.clone();
        glib::timeout_add_local_once(Duration::from_millis(16), move || {
            revealer.set_reveal_child(true);
        });
    });

    let revealer_on_closed = revealer.clone();
    popover.connect_closed(move |_| {
        revealer_on_closed.set_reveal_child(false);
    });
}

pub(crate) fn set_placement(
    popover: &gtk::Popover,
    revealer: &gtk::Revealer,
    edge: BarEdge,
    region: BarRegion,
) {
    popover.set_position(position_for_edge(edge));
    let (x_offset, y_offset) = offset_for_placement(edge, region);
    popover.set_offset(x_offset, y_offset);
    popover.set_margin_start(margin_start_for_placement(edge, region));
    popover.set_margin_end(margin_end_for_placement(edge, region));
    popover.set_margin_top(margin_top_for_placement(edge, region));
    popover.set_margin_bottom(margin_bottom_for_placement(edge, region));
    revealer.set_transition_type(transition_for_edge(edge));
}

pub(crate) fn position_for_edge(edge: BarEdge) -> gtk::PositionType {
    match edge {
        BarEdge::Top => gtk::PositionType::Bottom,
        BarEdge::Bottom => gtk::PositionType::Top,
        BarEdge::Left => gtk::PositionType::Right,
        BarEdge::Right => gtk::PositionType::Left,
    }
}

fn offset_for_placement(edge: BarEdge, _region: BarRegion) -> (i32, i32) {
    match edge {
        BarEdge::Top => (0, DROPDOWN_GAP),
        BarEdge::Bottom => (0, -DROPDOWN_GAP),
        BarEdge::Left => (DROPDOWN_GAP, 0),
        BarEdge::Right => (-DROPDOWN_GAP, 0),
    }
}

pub(crate) fn x_offset_for_placement(edge: BarEdge, region: BarRegion) -> i32 {
    offset_for_placement(edge, region).0
}

pub(crate) fn y_offset_for_placement(edge: BarEdge, region: BarRegion) -> i32 {
    offset_for_placement(edge, region).1
}

pub(crate) fn margin_start_for_placement(edge: BarEdge, region: BarRegion) -> i32 {
    if matches!(edge, BarEdge::Top | BarEdge::Bottom) && region == BarRegion::Start {
        DROPDOWN_GAP
    } else {
        0
    }
}

pub(crate) fn margin_end_for_placement(edge: BarEdge, region: BarRegion) -> i32 {
    if matches!(edge, BarEdge::Top | BarEdge::Bottom) && region == BarRegion::End {
        DROPDOWN_GAP
    } else {
        0
    }
}

pub(crate) fn margin_top_for_placement(edge: BarEdge, region: BarRegion) -> i32 {
    if matches!(edge, BarEdge::Left | BarEdge::Right) && region == BarRegion::Start {
        DROPDOWN_GAP
    } else {
        0
    }
}

pub(crate) fn margin_bottom_for_placement(edge: BarEdge, region: BarRegion) -> i32 {
    if matches!(edge, BarEdge::Left | BarEdge::Right) && region == BarRegion::End {
        DROPDOWN_GAP
    } else {
        0
    }
}

pub(crate) fn transition_for_edge(edge: BarEdge) -> gtk::RevealerTransitionType {
    match edge {
        BarEdge::Top => gtk::RevealerTransitionType::SlideDown,
        BarEdge::Bottom => gtk::RevealerTransitionType::SlideUp,
        BarEdge::Left => gtk::RevealerTransitionType::SlideRight,
        BarEdge::Right => gtk::RevealerTransitionType::SlideLeft,
    }
}

#[cfg(test)]
#[path = "dropdown_test.rs"]
mod tests;
