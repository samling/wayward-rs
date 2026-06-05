use relm4::gtk;
use relm4::gtk::prelude::{BoxExt, ToggleButtonExt, WidgetExt};
use std::sync::Arc;
use wayle_power_profiles::PowerProfilesService;
use wayle_power_profiles::types::profile::PowerProfile;

pub(super) struct BatteryDropdown {
    pub(super) root: gtk::Box,
    pub(super) meter: gtk::LevelBar,
    pub(super) percentage_label: gtk::Label,
    pub(super) energy_rate_label: gtk::Label,
    pub(super) health_label: gtk::Label,
    pub(super) profile_buttons: Vec<(PowerProfile, gtk::ToggleButton)>,
}

pub(super) fn battery_dropdown_content(
    power_profiles: Option<Arc<PowerProfilesService>>,
) -> BatteryDropdown {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.add_css_class("dropdown-content");
    root.add_css_class("battery-dropdown-content");

    let title = gtk::Label::new(Some("Battery"));
    title.add_css_class("dropdown-title");
    title.add_css_class("battery-dropdown-title");
    title.set_halign(gtk::Align::Start);
    root.append(&title);

    let summary = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    summary.add_css_class("battery-summary");
    summary.set_hexpand(true);

    let meter = gtk::LevelBar::for_interval(0.0, 100.0);
    meter.add_css_class("battery-meter");
    meter.set_hexpand(true);
    meter.set_value(0.0);
    summary.append(&meter);

    let percentage_label = gtk::Label::new(Some("NaN"));
    percentage_label.add_css_class("battery-dropdown-percentage");
    percentage_label.set_halign(gtk::Align::End);
    summary.append(&percentage_label);

    root.append(&summary);

    let details = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    details.add_css_class("battery-details");
    details.set_hexpand(true);
    details.set_homogeneous(true);

    let (energy_rate, energy_rate_label) = detail_item("Energy rate", "");
    details.append(&energy_rate);

    let (health, health_label) = detail_item("Health", "");
    details.append(&health);

    root.append(&details);

    let profile_title = gtk::Label::new(Some("Power profile"));
    profile_title.add_css_class("dropdown-title");
    profile_title.add_css_class("battery-dropdown-title");
    profile_title.set_halign(gtk::Align::Start);
    root.append(&profile_title);

    let profiles = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    profiles.add_css_class("profile-segments");
    profiles.set_homogeneous(true);

    let saver = profile_button(
        "Power Saver",
        PowerProfile::PowerSaver,
        power_profiles.clone(),
    );
    saver.add_css_class("power-saver");
    profiles.append(&saver);

    let balanced = profile_button("Balanced", PowerProfile::Balanced, power_profiles.clone());
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

    BatteryDropdown {
        root,
        meter,
        percentage_label,
        energy_rate_label,
        health_label,
        profile_buttons: vec![
            (PowerProfile::PowerSaver, saver),
            (PowerProfile::Balanced, balanced),
            (PowerProfile::Performance, performance),
        ],
    }
}

fn detail_item(label: &str, value: &str) -> (gtk::Box, gtk::Label) {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 2);
    root.add_css_class("battery-detail");
    root.set_hexpand(true);

    let title = gtk::Label::new(Some(label));
    title.add_css_class("battery-detail-label");
    title.set_halign(gtk::Align::Start);
    root.append(&title);

    let value = gtk::Label::new(Some(value));
    value.add_css_class("battery-detail-value");
    value.set_halign(gtk::Align::Start);
    root.append(&value);

    (root, value)
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

