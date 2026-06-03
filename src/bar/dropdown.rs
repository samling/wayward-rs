use gtk::prelude::*;
use relm4::gtk;

use crate::bar::layout::BarEdge;

#[derive(Clone)]
pub(crate) struct Dropdown {
    popover: gtk::Popover,
}

impl Dropdown {
    pub(crate) fn new(class_name: &str) -> Self {
        let popover = gtk::Popover::new();
        popover.set_has_arrow(false);
        popover.set_autohide(true);
        popover.add_css_class("dropdown");
        popover.add_css_class(class_name);

        Self { popover }
    }

    pub(crate) fn menu_button(
        class_name: &str,
        edge: BarEdge,
        button_child: &impl IsA<gtk::Widget>,
        popover_child: &impl IsA<gtk::Widget>,
    ) -> (gtk::MenuButton, Self) {
        let button = gtk::MenuButton::new();
        button.set_always_show_arrow(false);
        button_child.add_css_class("bar-item-content");
        button.set_child(Some(button_child));

        crate::bar::style::add_bar_item_classes(&button, class_name);
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
        self.popover.set_child(Some(child));
        button.set_popover(Some(&self.popover));
    }

    pub(crate) fn set_edge(&self, edge: BarEdge) {
        self.popover.set_position(position_for_edge(edge));
        self.set_position_class(edge);
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
