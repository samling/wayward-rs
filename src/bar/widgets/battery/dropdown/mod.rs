use crate::bar::{dropdown, layout::BarEdge, widget::BarRegion};
use relm4::gtk;
use relm4::gtk::prelude::ButtonExt;
use relm4::gtk::prelude::ToggleButtonExt;
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::prelude::*;
use std::sync::Arc;
use wayle_power_profiles::PowerProfilesService;
use wayle_power_profiles::types::profile::PowerProfile;

use super::history::{
    CHARGE_HISTORY_WINDOW_SECONDS, graph_points, load_charge_history, recent_points,
};
use super::history_graph::BatteryHistoryGraph;
use super::view_model::BatteryViewModel;

pub(super) struct BatteryDropdownInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
    pub(super) power_profiles: Option<Arc<PowerProfilesService>>,
}

pub(super) struct BatteryDropdown {
    view_model: BatteryViewModel,
    edge: BarEdge,
    region: BarRegion,
    power_profiles: Option<Arc<PowerProfilesService>>,
    active_profile: Option<PowerProfile>,
    available_profiles: Vec<PowerProfile>,
    history_graph: BatteryHistoryGraph,
    shell: Option<dropdown::DropdownPopover>,
}

#[derive(Debug)]
pub(super) enum BatteryDropdownInput {
    SetPlacement {
        edge: BarEdge,
        region: BarRegion,
    },
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
    type Init = BatteryDropdownInit;
    type Input = BatteryDropdownInput;
    type Output = ();

    view! {
        #[root]
        #[template]
        #[name = "shell"]
        dropdown::DropdownPopover(dropdown::DropdownPopoverInit {
            root_css_class: "battery-dropdown",
            content_css_class: "battery-dropdown-content",
            content_spacing: 8,
        }) {
            #[template_child]
            content {

                    gtk::Box {
                        add_css_class: "dropdown-header",
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

                        gtk::Box {
                            add_css_class: "battery-detail",
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 2,
                            set_hexpand: true,

                            gtk::Label {
                                add_css_class: "battery-detail-label",
                                set_halign: gtk::Align::Start,

                                #[watch]
                                set_text: model.view_model.time_remaining_label,
                            },

                            gtk::Label {
                                add_css_class: "battery-detail-value",
                                set_halign: gtk::Align::Start,

                                #[watch]
                                set_text: &model.view_model.time_remaining_text,
                            },
                        },
                    },

                    gtk::Label {
                        add_css_class: "dropdown-title",
                        add_css_class: "battery-dropdown-title",
                        set_halign: gtk::Align::Start,
                        set_text: "Charge history",
                    },

                    #[local_ref]
                    history_graph -> gtk::DrawingArea {},

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
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let history_graph = BatteryHistoryGraph::new();
        let mut model = Self {
            view_model: BatteryViewModel::unavailable(),
            edge: init.edge,
            region: init.region,
            power_profiles: init.power_profiles,
            active_profile: None,
            available_profiles: Vec::new(),
            history_graph,
            shell: None,
        };

        refresh_history_graph(&model.history_graph);

        let history_graph = model.history_graph.root();
        let widgets = view_output!();

        root.set_placement(init.edge, init.region);
        root.connect_revealer();
        model.shell = Some(root.clone());

        let history_graph_on_map = model.history_graph.clone();
        root.as_ref().connect_map(move |_| {
            refresh_history_graph(&history_graph_on_map);
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            BatteryDropdownInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
                if let Some(shell) = &self.shell {
                    shell.set_placement(edge, region);
                }
            }
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

fn refresh_history_graph(history_graph: &BatteryHistoryGraph) {
    match load_charge_history() {
        Ok(history) => {
            let recent_history = recent_points(&history, CHARGE_HISTORY_WINDOW_SECONDS);
            let graph = graph_points(&recent_history);

            tracing::debug!(
                history_points = history.len(),
                recent_points = recent_history.len(),
                graph_points = graph.len(),
                "Loaded battery charge history"
            );

            history_graph.set_points(graph);
        }
        Err(error) => {
            tracing::warn!(?error, "Failed to load battery charge history");
            history_graph.set_points(Vec::new());
        }
    }
}
