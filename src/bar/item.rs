use relm4::gtk::{self, prelude::WidgetExt};

pub(super) fn style_label(label: &gtk::Label, class_name: &str) {
    label.add_css_class("bar-item");
    label.add_css_class(class_name);
}
