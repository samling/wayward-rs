use relm4::gtk::prelude::WidgetExt;

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
