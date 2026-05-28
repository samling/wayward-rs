use gtk::prelude::*;
use relm4::gtk;

use super::Bar;

impl Bar {
    pub(super) fn render_workspace_row(&self, row: &gtk::Box) {
        while let Some(child) = row.first_child() {
            row.remove(&child);
        }

        if let Some(status) = &self.status {
            let label = gtk::Label::new(Some(status));
            label.add_css_class("status");
            row.append(&label);
            return;
        }

        for workspace in &self.workspaces {
            let label = gtk::Label::new(Some(&workspace.label()));

            for class_name in workspace.css_classes() {
                label.add_css_class(class_name);
            }

            row.append(&label);
        }
    }
}
