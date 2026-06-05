use gtk::prelude::*;
use relm4::gtk;
use relm4::gtk::glib;
use std::time::Duration;

use crate::bar::layout::BarEdge;

const DROPDOWN_GAP: i32 = 6;

#[derive(Clone)]
pub(crate) struct Dropdown {
    popover: gtk::Popover,
    revealer: gtk::Revealer,
}

impl Dropdown {
    pub(crate) fn new(class_name: &str) -> Self {
        let popover = gtk::Popover::new();
        popover.set_has_arrow(false);
        popover.set_autohide(true);
        popover.add_css_class("dropdown");
        popover.add_css_class(class_name);

        let revealer = gtk::Revealer::new();
        revealer.set_transition_duration(140);
        revealer.set_reveal_child(false);

        Self { popover, revealer }
    }

    pub(crate) fn menu_button(
        class_name: &str,
        instance_class: Option<&str>,
        edge: BarEdge,
        button_child: &impl IsA<gtk::Widget>,
        popover_child: &impl IsA<gtk::Widget>,
    ) -> (gtk::MenuButton, Self) {
        let button = gtk::MenuButton::new();
        button.set_always_show_arrow(false);
        button_child.add_css_class("bar-item-content");
        button.set_child(Some(button_child));

        crate::bar::style::add_bar_item_classes(&button, class_name, instance_class);
        button.add_css_class("flat");

        let dropdown = Self::new(&format!("{class_name}-dropdown"));
        dropdown.bind_to_menu_button(&button, edge, popover_child);

        (button, dropdown)
    }

    pub(crate) fn bind_to_menu_button(
        &self,
        button: &gtk::MenuButton,
        edge: BarEdge,
        child: &impl IsA<gtk::Widget>,
    ) {
        self.set_edge(edge);
        self.revealer.set_child(Some(child));
        self.revealer.set_reveal_child(false);
        self.popover.set_child(Some(&self.revealer));
        button.set_popover(Some(&self.popover));

        let revealer = self.revealer.clone();
        self.popover.connect_map(move |_| {
            let revealer = revealer.clone();
            glib::timeout_add_local_once(Duration::from_millis(16), move || {
                revealer.set_reveal_child(true);
            });
        });

        let revealer = self.revealer.clone();
        self.popover.connect_closed(move |_| {
            revealer.set_reveal_child(false);
        });
    }

    pub(crate) fn set_edge(&self, edge: BarEdge) {
        self.popover.set_position(position_for_edge(edge));
        self.set_position_class(edge);
        self.popover.set_offset(0, offset_for_edge(edge));
        self.revealer.set_transition_type(transition_for_edge(edge));
    }

    fn set_position_class(&self, edge: BarEdge) {
        for class_name in [
            "position-top",
            "position-bottom",
            "position-left",
            "position-right",
        ] {
            self.popover.remove_css_class(class_name);
        }

        self.popover.add_css_class(match edge {
            BarEdge::Top => "position-bottom",
            BarEdge::Bottom => "position-top",
            BarEdge::Left => "position-right",
            BarEdge::Right => "position-left",
        });
    }
}

fn position_for_edge(edge: BarEdge) -> gtk::PositionType {
    match edge {
        BarEdge::Top => gtk::PositionType::Bottom,
        BarEdge::Bottom => gtk::PositionType::Top,
        BarEdge::Left => gtk::PositionType::Right,
        BarEdge::Right => gtk::PositionType::Left,
    }
}

fn offset_for_edge(edge: BarEdge) -> i32 {
    match edge {
        BarEdge::Top => DROPDOWN_GAP,
        BarEdge::Bottom => -DROPDOWN_GAP,
        BarEdge::Left | BarEdge::Right => 0,
    }
}

fn transition_for_edge(edge: BarEdge) -> gtk::RevealerTransitionType {
    match edge {
        BarEdge::Top => gtk::RevealerTransitionType::SlideDown,
        BarEdge::Bottom => gtk::RevealerTransitionType::SlideUp,
        BarEdge::Left => gtk::RevealerTransitionType::SlideRight,
        BarEdge::Right => gtk::RevealerTransitionType::SlideLeft,
    }
}