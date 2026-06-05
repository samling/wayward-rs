use crate::bar::dropdown::view as dropdown_view;
use crate::bar::state::BatterySnapshot;
use crate::bar::widget::{WidgetBuildContext, WidgetInstance};
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{BoxExt, ToggleButtonExt, WidgetExt};
use std::sync::Arc;
use wayle_power_profiles::PowerProfilesService;
use wayle_power_profiles::types::profile::PowerProfile;

use super::format::{
    battery_energy_rate_text, battery_health_text, battery_icon_name, battery_percentage_text, initial_text
};

pub(super) struct BatteryView {
    root: gtk::MenuButton,
    dropdown: crate::bar::dropdown::Dropdown,
    bar: BatteryBarView,
    dropdown_content: BatteryDropdownView,
}

impl BatteryView {
    pub(super) fn new(instance: &WidgetInstance, context: &WidgetBuildContext<'_>) -> Self {
        let bar = BatteryBarView::new(context.bar.edge.orientation());
        let dropdown_content = 
            BatteryDropdownView::new(context.services.power_profiles.clone());

        let instance_class = instance.instance_css_class();
        let (root, dropdown) = crate::bar::dropdown::Dropdown::menu_button(
            "battery",
            instance_class.as_deref(),
            context.bar.edge,
            &bar.root,
            &dropdown_content.root,
        );

        Self {
            root,
            dropdown,
            bar,
            dropdown_content,
        }
    }

    pub(super) fn root(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    pub(super) fn set_edge(&self, edge: crate::bar::layout::BarEdge) {
        self.dropdown.set_edge(edge);
    }

    pub(super) fn show_unavailable(&self) {
        self.bar.show_unavailable();
        self.dropdown_content.show_unavailable();
    }

    pub(super) fn show_snapshot(&self, snapshot: &BatterySnapshot) {
        self.bar.show_snapshot(snapshot);
        self.dropdown_content.show_snapshot(snapshot);
    }
}

struct BatteryBarView {
    root: gtk::Box,
    icon: gtk::Image,
    percentage_label: gtk::Label,
    energy_rate_label: gtk::Label,
}

impl BatteryBarView {
    fn new(orientation: gtk::Orientation) -> Self {
        let root = gtk::Box::new(orientation, 0);
        root.add_css_class("battery-content");

        let icon = gtk::Image::from_icon_name("battery-missing-symbolic");
        icon.add_css_class("battery-icon");
        root.append(&icon);

        let percentage_label = gtk::Label::new(Some(&initial_text()));
        percentage_label.add_css_class("battery-percentage");
        root.append(&percentage_label);

        let energy_rate_label = gtk::Label::new(None);
        energy_rate_label.add_css_class("battery-energy-rate");
        root.append(&energy_rate_label);

        Self {
            root,
            icon,
            percentage_label,
            energy_rate_label,
        }
    }

    fn show_unavailable(&self) {
        self.icon.set_icon_name(Some("battery-missing-symbolic"));
        self.percentage_label.set_text(&initial_text());
        self.energy_rate_label.set_text("");
    }

    fn show_snapshot(&self, snapshot: &BatterySnapshot) {
        self.icon.set_icon_name(Some(battery_icon_name(snapshot.percentage, snapshot.state)));
        self.percentage_label.set_text(&battery_percentage_text(snapshot.percentage));
        self.energy_rate_label.set_text(&battery_energy_rate_text(snapshot.energy_rate));
    }
}

struct BatteryDropdownView {
    root: gtk::Box,
    meter: gtk::LevelBar,
    percentage_label: gtk::Label,
    energy_rate_label: gtk::Label,
    health_label: gtk::Label,
    profile_buttons: Vec<(PowerProfile, gtk::ToggleButton)>,
}

impl BatteryDropdownView {
    fn new(power_profiles: Option<Arc<PowerProfilesService>>) -> Self {
        let root = dropdown_view::content("battery-dropdown-content");
        let title = dropdown_view::title("Battery", "battery-dropdown-title");
        root.append(&title);

        let summary = dropdown_view::row("battery-summary", 8);

        let meter = gtk::LevelBar::for_interval(0.0, 100.0);
        meter.add_css_class("battery-meter");
        meter.set_hexpand(true);
        meter.set_value(0.0);
        summary.append(&meter);

        let percentage_label = gtk::Label::new(Some(&initial_text()));
        percentage_label.add_css_class("battery-dropdown-percentage");
        percentage_label.set_halign(gtk::Align::End);
        summary.append(&percentage_label);

        root.append(&summary);

        let details = dropdown_view::homogeneous_row("battery-details", 8);

        let energy_rate = dropdown_view::detail_item(
            "Energy rate",
            "",
            "battery-detail",
            "battery-detail-label",
            "battery-detail-value",
        );
        details.append(&energy_rate.root);

        let health = dropdown_view::detail_item(
            "Health",
            "",
            "battery-detail",
            "battery-detail-label",
            "battery-detail-value",
        );
        details.append(&health.root);

        root.append(&details);

        let profile_title = dropdown_view::title("Power profile", "battery-dropdown-title");
        root.append(&profile_title);

        let profiles = dropdown_view::segmented_row("profile-segments");

        let saver = profile_button(
            "Power Saver",
            PowerProfile::PowerSaver,
            power_profiles.clone(),
        );
        saver.add_css_class("power-saver");
        profiles.append(&saver);
 
        let balanced = profile_button(
            "Balanced",
            PowerProfile::Balanced,
            power_profiles.clone(),
        );
        balanced.add_css_class("balanced");
        balanced.set_group(Some(&saver));
        profiles.append(&balanced);

        let performance = profile_button(
            "Performance",
            PowerProfile::Performance,
            power_profiles.clone(),
        );
        performance.add_css_class("performance");
        performance.set_group(Some(&saver));
        profiles.append(&performance);

        root.append(&profiles);

        Self {
            root,
            meter,
            percentage_label,
            energy_rate_label: energy_rate.value,
            health_label: health.value,
            profile_buttons: vec![
                (PowerProfile::PowerSaver, saver),
                (PowerProfile::Balanced, balanced),
                (PowerProfile::Performance, performance),
            ],
        }
    }

    fn show_unavailable(&self) {
        self.meter.set_value(0.0);
        self.percentage_label.set_text(&initial_text());
        self.energy_rate_label.set_text("");
        self.health_label.set_text("");

        for (_, button) in &self.profile_buttons {
            button.set_sensitive(false);
            button.set_active(false);
        }
    }

    fn show_snapshot(&self, snapshot: &BatterySnapshot) {
        self.meter.set_value(snapshot.percentage.clamp(0.0, 100.0));
        self.percentage_label.set_text(&battery_percentage_text(snapshot.percentage));
        self.energy_rate_label.set_text(&battery_energy_rate_text(snapshot.energy_rate));
        self.health_label.set_text(&battery_health_text(snapshot.capacity));

        for (profile, button) in &self.profile_buttons {
            let available = snapshot.available_profiles.contains(profile);
            button.set_sensitive(available);
            button.set_active(snapshot.active_profile == Some(*profile));
        }
    }
}

fn profile_button(
    label: &str,
    profile: PowerProfile,
    power_profiles: Option<Arc<PowerProfilesService>>,
) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::with_label(label);
    button.add_css_class("profile-button");
    button.set_sensitive(false);

    button.connect_toggled(move |button| {
        if !button.is_active() {
            return;
        }

        let Some(service) = power_profiles.clone() else {
            return;
        };

        relm4::spawn(async move {
            if let Err(error) = service.power_profiles.set_active_profile(profile).await {
                tracing::error!("Failed to set power profile: {error}");
            }
        });
    });

    button
}