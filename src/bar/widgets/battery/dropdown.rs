use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{BoxExt, ToggleButtonExt, WidgetExt};
use std::sync::Arc;
use wayle_power_profiles::PowerProfilesService;
use wayle_power_profiles::types::profile::PowerProfile;

pub(super) fn battery_dropdown_content(
    power_profiles: Option<Arc<PowerProfilesService>>,
) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.add_css_class("dropdown-content");
    root.add_css_class("battery-dropdown-content");

    let title = gtk::Label::new(Some("Power profile"));
    title.add_css_class("dropdown-title");
    title.add_css_class("battery-dropdown-title");
    title.set_halign(gtk::Align::Start);
    root.append(&title);

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
    root
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

pub(super) fn profile_buttons(root: &gtk::Box) -> Vec<(PowerProfile, gtk::ToggleButton)> {
    let Some(profiles_widget) = root.last_child() else {
        return Vec::new();
    };

    let Ok(profiles) = profiles_widget.downcast::<gtk::Box>() else {
        return Vec::new();
    };

    let mut buttons = Vec::new();

    if let Some(button) = profiles
        .first_child()
        .and_then(|child| child.downcast().ok())
    {
        buttons.push((PowerProfile::PowerSaver, button));
    }

    if let Some(button) = profiles
        .first_child()
        .and_then(|child| child.next_sibling())
        .and_then(|child| child.downcast().ok())
    {
        buttons.push((PowerProfile::Balanced, button));
    }

    if let Some(button) = profiles
        .last_child()
        .and_then(|child| child.downcast().ok())
    {
        buttons.push((PowerProfile::Performance, button));
    }

    buttons
}
