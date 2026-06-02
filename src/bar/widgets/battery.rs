use crate::bar::BarMsg;
use crate::bar::state::{BarItemState, BatterySnapshot, BatteryState};
use crate::bar::widget::{BarContext, BarWidget, BarWidgetRuntime, WidgetInstance};
use crate::shell::ShellMsg;
use futures::{StreamExt, select};
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{ToggleButtonExt, WidgetExt};
use std::sync::Arc;
use wayle_battery::BatteryService;
use wayle_battery::types::DeviceState;
use wayle_power_profiles::PowerProfilesService;
use wayle_power_profiles::types::profile::PowerProfile;

struct BatteryRuntime {
    root: gtk::MenuButton,
    label: gtk::Label,
    dropdown: crate::bar::dropdown::Dropdown,
    profile_buttons: Vec<(PowerProfile, gtk::ToggleButton)>,
}

impl BarWidgetRuntime for BatteryRuntime {
    fn root(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, context: &BarContext) {
        self.dropdown.set_edge(context.edge);

        let snapshot = match state {
            BarItemState::Battery(BatteryState::Ready(snapshot)) => snapshot,
            BarItemState::Battery(BatteryState::Unavailable) => {
                self.label.set_text(&initial_text());

                for (_, button) in &self.profile_buttons {
                    button.set_sensitive(false);
                    button.set_active(false);
                }

                return;
            }
            _ => return,
        };

        let text = battery_text(snapshot.percentage, snapshot.state);
        self.label.set_text(&text);

        for (profile, button) in &self.profile_buttons {
            let available = snapshot.available_profiles.contains(profile);
            button.set_sensitive(available);
            button.set_active(snapshot.active_profile == Some(*profile));
        }
    }
}

pub(crate) struct BatteryWidget;

impl BarWidget for BatteryWidget {
    fn id(&self) -> &'static str {
        "battery"
    }

    fn build(
        &self,
        _instance: &WidgetInstance,
        _sender: &relm4::Sender<BarMsg>,
        services: &crate::services::ShellServices,
    ) -> Box<dyn BarWidgetRuntime> {
        let label = gtk::Label::new(Some(&initial_text()));
        label.add_css_class("battery-label");

        let power_profiles = services.power_profiles.clone();
        let dropdown_content = battery_dropdown_content(power_profiles.clone());
        let profile_buttons = profile_buttons(&dropdown_content);

        let (root, dropdown) = crate::bar::dropdown::Dropdown::menu_button(
            "battery",
            crate::bar::layout::BarEdge::Top,
            &label,
            &dropdown_content,
        );

        Box::new(BatteryRuntime {
            root,
            label,
            dropdown,
            profile_buttons,
        })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Battery(BatteryState::Unavailable))
    }

    fn start(
        &self,
        sender: Sender<ShellMsg>,
        services: &crate::services::ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        Some(start(
            sender,
            services.battery.clone(),
            services.power_profiles.clone(),
        ))
    }
}

pub(super) fn initial_text() -> String {
    "NaN".to_string()
}

pub(crate) fn start(
    sender: Sender<ShellMsg>,
    service: Option<Arc<BatteryService>>,
    power_profiles: Option<Arc<PowerProfilesService>>,
) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_battery_watcher(sender, service, power_profiles).await;
    })
}

fn battery_text(percentage: f64, state: DeviceState) -> String {
    format!("{percentage:.0}% {state}")
}

fn battery_dropdown_content(power_profiles: Option<Arc<PowerProfilesService>>) -> gtk::Box {
    use relm4::gtk::prelude::{BoxExt, WidgetExt};

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

    let saver = profile_button("Power Saver", PowerProfile::PowerSaver, power_profiles.clone());
    saver.add_css_class("power-saver");
    profiles.append(&saver);

    let balanced = profile_button("Balanced", PowerProfile::Balanced, power_profiles.clone());
    balanced.add_css_class("balanced");
    balanced.set_group(Some(&saver));
    profiles.append(&balanced);

    let performance = profile_button("Performance", PowerProfile::Performance, power_profiles.clone());
    performance.add_css_class("performance");
    performance.set_group(Some(&saver));
    profiles.append(&performance);

    root.append(&profiles);
    root
}

fn profile_button(label: &str, profile: PowerProfile, power_profiles: Option<Arc<PowerProfilesService>>) -> gtk::ToggleButton {
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

fn profile_buttons(root: &gtk::Box) -> Vec<(PowerProfile, gtk::ToggleButton)> {
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

async fn run_battery_watcher(
    sender: Sender<ShellMsg>,
    service: Option<Arc<BatteryService>>,
    power_profiles: Option<Arc<PowerProfilesService>>,
) {
    let Some(service) = service else {
        let _ = sender.send(battery_message(BatteryState::Unavailable));
        return;
    };

    send_battery_snapshot(&sender, service.as_ref(), power_profiles.as_deref());

    let mut percentage_updates = service.device.percentage.watch().fuse();
    let mut state_updates = service.device.state.watch().fuse();
    // let mut profile_updates = power_profiles.as_ref().map(|service| service.power_profiles.active_profile.watch().fuse());
    // let mut profiles_update = power_profiles.as_ref().map(|service| service.power_profiles.profiles.watch().fuse());

    loop {
        select! {
            update = percentage_updates.next() => {
                if update.is_none() {
                    break;
                }

                send_battery_snapshot(&sender, &service, power_profiles.as_deref());
            }
            update = state_updates.next() => {
                if update.is_none() {
                    break;
                }
                send_battery_snapshot(&sender, &service, power_profiles.as_deref());
            }
        }
    }

    let _ = sender.send(battery_message(BatteryState::Unavailable));
}

fn send_battery_snapshot(
    sender: &Sender<ShellMsg>,
    service: &BatteryService,
    power_profiles: Option<&PowerProfilesService>,
) {
    let percentage = service.device.percentage.get();
    let state = service.device.state.get();

    let active_profile = power_profiles.map(|service| service.power_profiles.active_profile.get());

    let available_profiles = power_profiles
        .map(|service| {
            service
                .power_profiles
                .profiles
                .get()
                .into_iter()
                .map(|profile| profile.profile)
                .filter(|profile| *profile != PowerProfile::Unknown)
                .collect()
        })
        .unwrap_or_default();

    let snapshot = BatterySnapshot {
        percentage,
        state,
        active_profile,
        available_profiles,
    };

    let _ = sender.send(battery_message(BatteryState::Ready(snapshot)));
}

fn battery_message(state: BatteryState) -> ShellMsg {
    ShellMsg::ItemStateChanged(BarItemState::Battery(state))
}
