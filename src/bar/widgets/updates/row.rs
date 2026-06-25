use relm4::factory::FactoryComponent;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

use super::model::{UpdatePackage, UpdateSeverity};

pub(super) struct UpdateRow {
    package: UpdatePackage,
}

#[derive(Debug)]
pub(super) enum UpdateRowInput {
    SetPackage(UpdatePackage),
}

#[relm4::factory(pub(super))]
impl FactoryComponent for UpdateRow {
    type Init = UpdatePackage;
    type Input = UpdateRowInput;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::Box {
            #[watch]
            set_css_classes: &self.root_classes(),
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,
            set_hexpand: true,

            gtk::Label {
                add_css_class: "updates-package-name",
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                #[watch]
                set_text: &self.package.name,
            },

            gtk::Label {
                add_css_class: "updates-version",
                set_halign: gtk::Align::End,
                #[watch]
                set_text: &version_text(&self.package),
            },
        }
    }

    fn init_model(
        package: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { package }
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            UpdateRowInput::SetPackage(package) => self.package = package,
        }
    }
}

impl UpdateRow {
    pub(super) fn name(&self) -> &str {
        &self.package.name
    }

    fn root_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["updates-row"];
        if let Some(severity) = severity_class(&self.package.severity) {
            classes.push(severity);
        }
        classes
    }
}

fn severity_class(severity: &UpdateSeverity) -> Option<&'static str> {
    match severity {
        UpdateSeverity::Critical => Some("critical"),
        UpdateSeverity::Warning => Some("warning"),
        UpdateSeverity::Normal => None,
    }
}

fn version_text(package: &UpdatePackage) -> String {
    format!("{} -> {}", package.old_version, package.new_version)
}

#[cfg(test)]
#[path = "row_test.rs"]
mod tests;
