use relm4::gtk;
use relm4::gtk::prelude::{BoxExt, WidgetExt};

pub(crate) struct DetailItem {
    pub(crate) root: gtk::Box,
    pub(crate) value: gtk::Label,
}

pub(crate) fn content(class_name: &str) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.add_css_class("dropdown-content");
    root.add_css_class(class_name);
    root
}

pub(crate) fn title(text: &str, class_name: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("dropdown-title");
    label.add_css_class(class_name);
    label.set_halign(gtk::Align::Start);
    label
}

pub(crate) fn row(class_name: &str, spacing: i32) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, spacing);
    root.add_css_class(class_name);
    root.set_hexpand(true);
    root
}

pub(crate) fn homogeneous_row(class_name: &str, spacing: i32) -> gtk::Box {
    let root = row(class_name, spacing);
    root.set_homogeneous(true);
    root
}

pub(crate) fn segmented_row(class_name: &str) -> gtk::Box {
    let root = row(class_name, 0);
    root.set_homogeneous(true);
    root
}

pub(crate) fn detail_item(
    label_text: &str,
    value_text: &str,
    item_class: &str,
    label_class: &str,
    value_class: &str,
) -> DetailItem {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 2);
    root.add_css_class("dropdown-detail");
    root.add_css_class(item_class);
    root.set_hexpand(true);

    let label = gtk::Label::new(Some(label_text));
    label.add_css_class("dropdown-detail-label");
    label.add_css_class(label_class);
    label.set_halign(gtk::Align::Start);
    root.append(&label);

    let value = gtk::Label::new(Some(value_text));
    value.add_css_class("dropdown-detail-value");
    value.add_css_class(value_class);
    value.set_halign(gtk::Align::Start);
    root.append(&value);

    DetailItem { root, value }
}