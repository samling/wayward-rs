use gtk::prelude::*;
use relm4::gtk;
use relm4::gtk::glib;
use std::time::Duration;

use crate::bar::layout::BarEdge;

const DROPDOWN_GAP: i32 = 6;
pub(crate) const TRANSITION_MS: u32 = 140;

pub(crate) fn install_revealer(popover: &gtk::Popover, revealer: &gtk::Revealer, edge: BarEdge) {
    revealer.set_transition_duration(TRANSITION_MS);
    revealer.set_reveal_child(false);

    if let Some(child) = popover.child() {
        popover.set_child(None::<&gtk::Widget>);
        revealer.set_child(Some(&child));
    }

    popover.set_child(Some(revealer));
    set_edge(popover, revealer, edge);

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

pub(crate) fn set_edge(popover: &gtk::Popover, revealer: &gtk::Revealer, edge: BarEdge) {
    popover.set_position(position_for_edge(edge));
    popover.set_offset(0, offset_for_edge(edge));
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

pub(crate) fn offset_for_edge(edge: BarEdge) -> i32 {
    match edge {
        BarEdge::Top => DROPDOWN_GAP,
        BarEdge::Bottom => -DROPDOWN_GAP,
        BarEdge::Left | BarEdge::Right => 0,
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
