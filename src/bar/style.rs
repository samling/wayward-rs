use relm4::gtk::prelude::WidgetExt;

pub(super) fn add_bar_item_classes(widget: &impl WidgetExt, class_name: &str) {
    widget.add_css_class("bar-item");
    widget.add_css_class(class_name);
}
