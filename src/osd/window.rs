use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::gtk;

use super::OsdEvent;

pub(crate) struct OsdWindow {
    window: gtk::Window,
    root: gtk::Box,
    icon: gtk::Label,
    label: gtk::Label,
    level: gtk::LevelBar,
    hide_timeout: std::rc::Rc<std::cell::RefCell<Option<gtk::glib::SourceId>>>,
}

impl OsdWindow {
    pub(crate) fn new(monitor: &gtk::gdk::Monitor) -> Self {
        let window = gtk::Window::new();
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_monitor(Some(monitor));
        window.set_anchor(Edge::Bottom, true);
        window.set_margin(Edge::Bottom, 80);
        window.set_namespace(Some("wayward-osd"));
        window.set_visible(false);
        window.add_css_class("osd-window");

        let root = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        root.add_css_class("osd");

        let icon = gtk::Label::new(None);
        icon.add_css_class("osd-icon");

        let label = gtk::Label::new(None);
        label.add_css_class("osd-label");

        let level = gtk::LevelBar::new();
        level.set_min_value(0.0);
        level.set_max_value(100.0);
        level.set_width_request(180);
        level.add_css_class("osd-level");

        root.append(&icon);
        root.append(&label);
        root.append(&level);

        root.set_halign(gtk::Align::Center);
        root.set_valign(gtk::Align::Center);

        window.set_child(Some(&root));

        Self { window, root, icon, label, level, hide_timeout: std::rc::Rc::new(std::cell::RefCell::new(None)) }
    }

    pub(crate) fn show_event(&self, event: &OsdEvent) {
        self.root.remove_css_class("brightness");
        self.root.remove_css_class("volume");
        self.root.remove_css_class("muted");

        self.root.add_css_class(event.class_name());
        self.icon.set_text(event.icon());
        self.label.set_text(&event.label());
        self.level.set_value(event.percent().clamp(0.0, 100.0));

        self.window.present();
        self.reset_hide_timeout();
    }

    fn reset_hide_timeout(&self) {
        if let Some(timeout) = self.hide_timeout.borrow_mut().take() {
            timeout.remove();
        }

        let window = self.window.clone();
        let hide_timeout = self.hide_timeout.clone();

        let timeout = gtk::glib::timeout_add_local_once(
            std::time::Duration::from_millis(1500),
            move || {
                window.set_visible(false);
                hide_timeout.borrow_mut().take();
            },
        );

        *self.hide_timeout.borrow_mut() = Some(timeout);
    }
}