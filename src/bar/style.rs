use relm4::gtk::prelude::WidgetExt;
use relm4::gtk::{self, pango};

pub(crate) fn add_bar_item_classes(
    widget: &impl WidgetExt,
    class_name: &str,
    instance_class: Option<&str>,
) {
    widget.add_css_class("bar-item");
    widget.add_css_class(class_name);

    if let Some(instance_class) = instance_class {
        widget.add_css_class(instance_class);
    }
}

pub(crate) fn configure_bar_item_content(widget: &impl WidgetExt) {
    widget.set_halign(gtk::Align::Fill);
    widget.set_valign(gtk::Align::Fill);
}

pub(crate) fn configure_bar_label(label: &gtk::Label) {
    label.set_single_line_mode(true);
    label.set_yalign(0.5);

    let attributes = pango::AttrList::new();
    attributes.insert(pango::AttrFloat::new_line_height(1.0));
    label.set_attributes(Some(&attributes));
}
