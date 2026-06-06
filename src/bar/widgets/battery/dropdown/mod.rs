use std::sync::Arc;
use relm4::gtk::prelude::ButtonExt;
use relm4::gtk::prelude::PopoverExt;
use relm4::gtk::prelude::ToggleButtonExt;
use relm4::prelude::*;
use relm4::gtk;
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use wayle_power_profiles::PowerProfilesService;
use wayle_power_profiles::types::profile::PowerProfile;

use super::view_model::BatteryViewModel;

pub(super) struct BatteryDropdown {
    view_model: BatteryViewModel,
    power_profiles: Option<Arc<PowerProfilesService>>,
    active_profile: Option<PowerProfile>,
    available_profiles: Vec<PowerProfile>,
}

#[derive(Debug)]
pub(super) enum BatteryDropdownInput {
    SetViewModel(BatteryViewModel),
    SetSnapshot {
        view_model: BatteryViewModel,
        active_profile: Option<PowerProfile>,
        available_profiles: Vec<PowerProfile>,
    },
    SelectProfile(PowerProfile),
}

#[relm4::component(pub(super))]
impl SimpleComponent for BatteryDropdown {
    type Init = Option<Arc<PowerProfilesService>>;
    type Input = BatteryDropdownInput;
    type Output = ();

    view! {
        #[root]
        gtk::Popover {
            set_has_arrow: false,
            add_css_class: "dropdown",
            add_css_class: "battery-dropdown",

            gtk::Box {
                add_css_class: "dropdown-content",
                add_css_class: "battery-dropdown-content",
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8,

                gtk::Box {
                    add_css_class: "battery-dropdown-header",
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,
                    set_hexpand: true,

                    gtk::Label {
                        add_css_class: "dropdown-title",
                        add_css_class: "battery-dropdown-title",
                        set_halign: gtk::Align::Start,
                        set_hexpand: true,
                        set_text: "Battery",
                    },

                    gtk::Label {
                        add_css_class: "battery-state",
                        set_halign: gtk::Align::End,

                        #[watch]
                        set_text: &model.view_model.state_text,
                    }
                },

                gtk::Box {
                    add_css_class: "battery-summary",
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,
                    set_hexpand: true,

                    gtk::LevelBar {
                        add_css_class: "battery-meter",
                        set_hexpand: true,
                        set_min_value: 0.0,
                        set_max_value: 100.0,

                        #[watch]
                        set_value: model.view_model.meter_value,
                    },

                    gtk::Label {
                        add_css_class: "battery-dropdown-percentage",
                        set_halign: gtk::Align::End,

                        #[watch]
                        set_text: &model.view_model.percentage_text,
                    },
                },

                gtk::Box {
                    add_css_class: "battery-details",
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 8,
                    set_hexpand: true,
                    set_homogeneous: true,

                    gtk::Box {
                        add_css_class: "battery-detail",
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 2,
                        set_hexpand: true,

                        gtk::Label {
                            add_css_class: "battery-detail-label",
                            set_halign: gtk::Align::Start,
                            set_text: "Energy rate",
                        },

                        gtk::Label {
                            add_css_class: "battery-detail-value",
                            set_halign: gtk::Align::Start,

                            #[watch]
                            set_text: &model.view_model.energy_rate_text,
                        },
                    },

                    gtk::Box {
                        add_css_class: "battery-detail",
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 2,
                        set_hexpand: true,

                        gtk::Label {
                            add_css_class: "battery-detail-label",
                            set_halign: gtk::Align::Start,
                            set_text: "Health",
                        },

                        gtk::Label {
                            add_css_class: "battery-detail-value",
                            set_halign: gtk::Align::Start,

                            #[watch]
                            set_text: &model.view_model.health_text,
                        },
                    },
                },

                gtk::Label {
                    add_css_class: "dropdown-title",
                    add_css_class: "battery-dropdown-title",
                    set_halign: gtk::Align::Start,
                    set_text: "Power profile",
                },

                gtk::Box {
                    add_css_class: "profile-segments",
                    set_orientation: gtk::Orientation::Horizontal,
                    set_homogeneous: true,

                    #[name = "saver_button"]
                    gtk::ToggleButton {
                        add_css_class: "profile-button",
                        add_css_class: "power-saver",

                        #[watch]
                        set_sensitive: model.has_profile(PowerProfile::PowerSaver),

                        connect_toggled[sender] => move |button| {
                            if button.is_active() {
                                sender.input(BatteryDropdownInput::SelectProfile(PowerProfile::PowerSaver))
                            }
                        } @saver_handler,

                        #[watch]
                        #[block_signal(saver_handler)]
                        set_active: model.is_active_profile(PowerProfile::PowerSaver),

                        set_label: "Power Saver",
                    },

                    gtk::ToggleButton {
                        add_css_class: "profile-button",
                        add_css_class: "balanced",
                        set_group: Some(&saver_button),

                        #[watch]
                        set_sensitive: model.has_profile(PowerProfile::Balanced),

                        connect_toggled[sender] => move |button| {
                            if button.is_active() {
                                sender.input(BatteryDropdownInput::SelectProfile(PowerProfile::Balanced))
                            }
                        } @balanced_handler,

                        #[watch]
                        #[block_signal(balanced_handler)]
                        set_active: model.is_active_profile(PowerProfile::Balanced)
                            && model.is_active_profile(PowerProfile::Balanced),

                        set_label: "Balanced",
                    },


                    gtk::ToggleButton {
                        add_css_class: "profile-button",
                        add_css_class: "performance",
                        set_group: Some(&saver_button),

                        #[watch]
                        set_sensitive: model.has_profile(PowerProfile::Performance),

                        connect_toggled[sender] => move |button| {
                            if button.is_active() {
                                sender.input(BatteryDropdownInput::SelectProfile(PowerProfile::Performance))
                            }
                        } @performance_handler,

                        #[watch]
                        #[block_signal(performance_handler)]
                        set_active: model.is_active_profile(PowerProfile::Performance)
                            && model.is_active_profile(PowerProfile::Performance),

                        set_label: "Performance",
                    },
                },
            },
        },
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            view_model: BatteryViewModel::unavailable(),
            power_profiles: init,
            active_profile: None,
            available_profiles: Vec::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            BatteryDropdownInput::SetViewModel(view_model) => {
                self.view_model = view_model;
                self.active_profile = None;
                self.available_profiles.clear();
            }
            BatteryDropdownInput::SetSnapshot {
                view_model,
                active_profile,
                available_profiles,
            } => {
                self.view_model = view_model;
                self.active_profile = active_profile;
                self.available_profiles = available_profiles;
            }
            BatteryDropdownInput::SelectProfile(profile) => {
                if !self.has_profile(profile) || self.is_active_profile(profile) {
                    return;
                }

                let Some(service) = self.power_profiles.clone() else {
                    return;
                };

                relm4::spawn_local(async move {
                    if let Err(error) = service.power_profiles.set_active_profile(profile).await {
                        tracing::error!(?error, ?profile, "Failed to set power profile");
                    }
                });
            }
        }
    }
}

impl BatteryDropdown {
    fn has_profile(&self, profile: PowerProfile) -> bool {
        self.available_profiles.contains(&profile)
    }

    fn is_active_profile(&self, profile: PowerProfile) -> bool {
        self.active_profile == Some(profile)
    }
}