use relm4::factory::FactoryComponent;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

use super::model::{UpdatePackage, UpdateSeverity};

pub(super) struct UpdateRow {
    package: UpdatePackage,
}

#[relm4::factory(pub(super))]
impl FactoryComponent for UpdateRow {
    type Init = UpdatePackage;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "updates-row",
            add_css_class?: severity_class(&self.package.severity),
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,
            set_hexpand: true,

            gtk::Label {
                add_css_class: "updates-package-name",
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                set_text: &self.package.name,
            },

            gtk::Label {
                add_css_class: "updates-version",
                set_halign: gtk::Align::End,
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
}

impl UpdateRow {
    pub(super) fn name(&self) -> &str {
        &self.package.name
    }

    pub(super) fn set_package(&mut self, package: UpdatePackage) {
        self.package = package;
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
mod tests {
    use super::*;

    #[test]
    fn version_text_shows_old_and_new_versions() {
        let package = UpdatePackage {
            name: "linux".to_string(),
            old_version: "6.9.1.arch1-1".to_string(),
            new_version: "6.9.2.arch1-1".to_string(),
            severity: UpdateSeverity::Normal,
        };

        assert_eq!(version_text(&package), "6.9.1.arch1-1 -> 6.9.2.arch1-1");
    }
}
