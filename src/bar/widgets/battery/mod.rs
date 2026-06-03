mod dropdown;
mod format;
mod service;

use crate::bar::BarMsg;
use crate::bar::state::{BarItemState, BatteryState};
use crate::bar::widget::{BarContext, BarWidget, BarWidgetRuntime, WidgetInstance};
use crate::shell::ShellMsg;
use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{BoxExt, ToggleButtonExt, WidgetExt};
use wayle_power_profiles::types::profile::PowerProfile;

use self::dropdown::{battery_dropdown_content, profile_buttons};
use self::format::{
    battery_energy_rate_text, battery_icon_name, battery_percentage_text, initial_text,
};

struct BatteryRuntime {
    root: gtk::MenuButton,
    icon: gtk::Image,
    percentage_label: gtk::Label,
    energy_rate_label: gtk::Label,
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
                self.icon.set_icon_name(Some("battery-missing-symbolic"));
                self.percentage_label.set_text(&initial_text());
                self.energy_rate_label.set_text("");

                for (_, button) in &self.profile_buttons {
                    button.set_sensitive(false);
                    button.set_active(false);
                }

                return;
            }
            _ => return,
        };

        self.icon
            .set_icon_name(Some(battery_icon_name(snapshot.percentage, snapshot.state)));
        self.percentage_label
            .set_text(&battery_percentage_text(snapshot.percentage));
        self.energy_rate_label
            .set_text(&battery_energy_rate_text(snapshot.energy_rate));

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
        instance: &WidgetInstance,
        _sender: &relm4::Sender<BarMsg>,
        services: &crate::services::ShellServices,
        context: &BarContext,
    ) -> Box<dyn BarWidgetRuntime> {
        let content = gtk::Box::new(context.edge.orientation(), 0);
        content.add_css_class("battery-content");

        let icon = gtk::Image::from_icon_name("battery-missing-symbolic");
        icon.add_css_class("battery-icon");
        content.append(&icon);

        let percentage_label = gtk::Label::new(Some(&initial_text()));
        percentage_label.add_css_class("battery-percentage");
        content.append(&percentage_label);

        let energy_rate_label = gtk::Label::new(None);
        energy_rate_label.add_css_class("battery-energy-rate");
        content.append(&energy_rate_label);

        let power_profiles = services.power_profiles.clone();
        let dropdown_content = battery_dropdown_content(power_profiles.clone());
        let profile_buttons = profile_buttons(&dropdown_content);

        let instance_class = instance.instance_css_class();
        let (root, dropdown) = crate::bar::dropdown::Dropdown::menu_button(
            "battery",
            instance_class.as_deref(),
            context.edge,
            &content,
            &dropdown_content,
        );

        Box::new(BatteryRuntime {
            root,
            icon,
            percentage_label,
            energy_rate_label,
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
        Some(service::start(
            sender,
            services.battery.clone(),
            services.power_profiles.clone(),
        ))
    }
}
