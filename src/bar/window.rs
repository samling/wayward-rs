use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;

use super::layout::BarEdge;

pub(super) fn apply_size_hint(root: &gtk::ApplicationWindow, edge: BarEdge, size: i32) {
    let size = size.max(1);

    if edge.is_vertical() {
        root.set_size_request(size, -1);
        root.set_default_size(size, -1);
    } else {
        root.set_size_request(-1, size);
        root.set_default_size(-1, size);
    }
}

pub(super) fn configure_window(
    root: &gtk::ApplicationWindow,
    edge: BarEdge,
    size: i32,
    name: Option<&str>,
    monitor: Option<&gtk::gdk::Monitor>,
) {
    if !root.is_layer_window() {
        root.init_layer_shell();
    }

    root.set_monitor(monitor);
    root.set_layer(Layer::Top);

    clear_anchors(root);
    set_edge_anchors(root, edge);
    set_edge_classes(root, edge);
    apply_size_hint(root, edge, size);

    root.auto_exclusive_zone_enable();
    root.set_keyboard_mode(KeyboardMode::None);
    root.set_namespace(Some(name.unwrap_or("wayward")));
}

fn clear_anchors(root: &gtk::ApplicationWindow) {
    root.set_anchor(Edge::Top, false);
    root.set_anchor(Edge::Bottom, false);
    root.set_anchor(Edge::Left, false);
    root.set_anchor(Edge::Right, false);
}

fn set_edge_anchors(root: &gtk::ApplicationWindow, edge: BarEdge) {
    match edge {
        BarEdge::Top => {
            root.set_anchor(Edge::Top, true);
            root.set_anchor(Edge::Left, true);
            root.set_anchor(Edge::Right, true);
        }
        BarEdge::Bottom => {
            root.set_anchor(Edge::Bottom, true);
            root.set_anchor(Edge::Left, true);
            root.set_anchor(Edge::Right, true);
        }
        BarEdge::Left => {
            root.set_anchor(Edge::Left, true);
            root.set_anchor(Edge::Top, true);
            root.set_anchor(Edge::Bottom, true);
        }
        BarEdge::Right => {
            root.set_anchor(Edge::Right, true);
            root.set_anchor(Edge::Top, true);
            root.set_anchor(Edge::Bottom, true);
        }
    }
}

fn set_edge_classes(root: &gtk::ApplicationWindow, edge: BarEdge) {
    root.remove_css_class("horizontal");
    root.remove_css_class("vertical");

    if edge.is_vertical() {
        root.add_css_class("vertical");
    } else {
        root.add_css_class("horizontal");
    }
}
