use relm4::gtk;
use relm4::gtk::prelude::*;

pub(crate) fn configure_root(
    root: &gtk::MenuButton,
    widget_class: &str,
    instance_class: Option<&str>,
) {
    root.set_always_show_arrow(false);
    root.set_cursor_from_name(Some("pointer"));
    crate::bar::style::add_bar_item_classes(root, widget_class, instance_class);
    root.add_css_class("flat");
}

pub(crate) fn content_box(
    orientation: gtk::Orientation,
    spacing: i32,
    content_class: &str,
) -> gtk::Box {
    let content = gtk::Box::new(orientation, spacing);
    crate::bar::style::add_bar_item_content_classes(&content, content_class);
    content
}

pub(crate) fn attach_content(root: &gtk::MenuButton, content: &gtk::Box) {
    root.set_child(Some(content));
}

pub(crate) fn attach_popover(root: &gtk::MenuButton, popover: &impl IsA<gtk::Popover>) {
    root.set_popover(Some(popover));
}
