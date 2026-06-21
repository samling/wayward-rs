#![allow(deprecated)]

use std::cell::Cell;
use std::rc::Rc;

use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

use crate::bar::{dropdown, layout::BarEdge, widget::BarRegion};

use super::model::{AudioDeviceSummary, VolumeSnapshot};

pub(super) struct VolumeDropdown {
    edge: BarEdge,
    region: BarRegion,
    snapshot: Option<VolumeSnapshot>,
    unavailable: Option<String>,
    syncing: Rc<Cell<bool>>,
}

pub(super) struct VolumeDropdownInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
}

#[derive(Debug)]
pub(super) enum VolumeDropdownInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetSnapshot(VolumeSnapshot),
    SetUnavailable(String),
    VolumeChanged(f64),
    ToggleMute,
    OutputSelected(u32),
    InputSelected(u32),
}

#[derive(Debug)]
pub(super) enum VolumeDropdownOutput {
    SetOutputVolume(f64),
    ToggleOutputMute,
    SetDefaultOutput(u32),
    SetDefaultInput(u32),
}

#[relm4::component(pub(super))]
impl Component for VolumeDropdown {
    type Init = VolumeDropdownInit;
    type Input = VolumeDropdownInput;
    type Output = VolumeDropdownOutput;
    type CommandOutput = ();

    view! {
        #[root]
        #[name = "popover"]
        gtk::Popover {
            set_has_arrow: false,
            set_autohide: true,
            add_css_class: "dropdown",
            add_css_class: "volume-dropdown",

            #[watch]
            set_position: dropdown::position_for_edge(model.edge),

            #[watch]
            set_offset: (
                dropdown::x_offset_for_placement(model.edge, model.region),
                dropdown::y_offset_for_placement(model.edge, model.region),
            ),

            #[watch]
            set_margin_start: dropdown::margin_start_for_placement(model.edge, model.region),
            #[watch]
            set_margin_end: dropdown::margin_end_for_placement(model.edge, model.region),
            #[watch]
            set_margin_top: dropdown::margin_top_for_placement(model.edge, model.region),
            #[watch]
            set_margin_bottom: dropdown::margin_bottom_for_placement(model.edge, model.region),

            #[name = "revealer"]
            gtk::Revealer {
                set_transition_duration: dropdown::TRANSITION_MS,
                set_reveal_child: false,

                #[watch]
                set_transition_type: dropdown::transition_for_edge(model.edge),

                gtk::Box {
                    add_css_class: "dropdown-content",
                    add_css_class: "volume-dropdown-content",
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    gtk::Box {
                        add_css_class: "dropdown-header",
                        add_css_class: "volume-dropdown-header",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk::Label {
                            add_css_class: "dropdown-title",
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                            set_text: "Volume",
                        },

                        #[name = "percent_label"]
                        gtk::Label {
                            add_css_class: "volume-dropdown-percent",
                            set_halign: gtk::Align::End,
                        },
                    },

                    #[name = "error_label"]
                    gtk::Label {
                        add_css_class: "volume-error",
                        set_halign: gtk::Align::Start,
                        set_visible: false,
                    },

                    gtk::Box {
                        add_css_class: "control-row",
                        add_css_class: "volume-control-row",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,

                        #[name = "mute_button"]
                        gtk::ToggleButton {
                            add_css_class: "control-toggle",
                            add_css_class: "icon-toggle",
                            set_cursor_from_name: Some("pointer"),
                            set_size_request: (32, 32),
                            set_tooltip_text: Some("Toggle mute"),

                            #[wrap(Some)]
                            set_child = &gtk::Image {
                                add_css_class: "control-toggle-icon",
                                set_icon_name: Some("audio-volume-high-symbolic"),
                            },
                        },

                        #[name = "volume_scale"]
                        gtk::Scale {
                            add_css_class: "control-scale",
                            set_hexpand: true,
                            set_orientation: gtk::Orientation::Horizontal,
                            set_draw_value: false,
                            set_range: (0.0, 100.0),
                            set_increments: (1.0, 10.0),
                        },
                    },

                    gtk::Box {
                        add_css_class: "control-row",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk::Label {
                            add_css_class: "control-label",
                            set_text: "Output",
                            set_halign: gtk::Align::Start,
                        },

                        #[name = "output_selector"]
                        gtk::ComboBoxText {
                            add_css_class: "control-combo",
                            set_hexpand: true,
                        },
                    },

                    gtk::Box {
                        add_css_class: "control-row",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk::Label {
                            add_css_class: "control-label",
                            set_text: "Input",
                            set_halign: gtk::Align::Start,
                        },

                        #[name = "input_selector"]
                        gtk::ComboBoxText {
                            add_css_class: "control-combo",
                            set_hexpand: true,
                        },
                    },
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            edge: init.edge,
            region: init.region,
            snapshot: None,
            unavailable: Some("Volume has not loaded yet".to_string()),
            syncing: Rc::new(Cell::new(false)),
        };

        let widgets = view_output!();

        dropdown::connect_revealer(&widgets.popover, &widgets.revealer);
        connect_controls(&widgets, &sender, model.syncing.clone());

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            VolumeDropdownInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
            }
            VolumeDropdownInput::SetSnapshot(snapshot) => {
                self.snapshot = Some(snapshot);
                self.unavailable = None;
                self.sync_widgets(widgets);
            }
            VolumeDropdownInput::SetUnavailable(error) => {
                self.snapshot = None;
                self.unavailable = Some(error);
                self.sync_widgets(widgets);
            }
            VolumeDropdownInput::VolumeChanged(percent) => {
                let _ = sender.output(VolumeDropdownOutput::SetOutputVolume(percent));
            }
            VolumeDropdownInput::ToggleMute => {
                let _ = sender.output(VolumeDropdownOutput::ToggleOutputMute);
            }
            VolumeDropdownInput::OutputSelected(key) => {
                let _ = sender.output(VolumeDropdownOutput::SetDefaultOutput(key));
            }
            VolumeDropdownInput::InputSelected(key) => {
                let _ = sender.output(VolumeDropdownOutput::SetDefaultInput(key));
            }
        }

        self.update_view(widgets, sender);
    }
}

impl VolumeDropdown {
    fn sync_widgets(&self, widgets: &VolumeDropdownWidgets) {
        self.syncing.set(true);

        match &self.snapshot {
            Some(snapshot) => {
                widgets.error_label.set_visible(false);
                widgets.volume_scale.set_sensitive(true);
                widgets.mute_button.set_sensitive(true);
                widgets.percent_label.set_text(&snapshot.percent_text());
                widgets.volume_scale.set_value(snapshot.display_percent());
                widgets.mute_button.set_active(snapshot.muted);
                set_mute_icon(&widgets.mute_button, snapshot.muted);

                sync_selector(
                    &widgets.output_selector,
                    &snapshot.outputs,
                    snapshot.default_output,
                );
                sync_selector(
                    &widgets.input_selector,
                    &snapshot.inputs,
                    snapshot.default_input,
                );
            }
            None => {
                let error = self.unavailable.as_deref().unwrap_or("Volume unavailable");
                widgets.error_label.set_text(error);
                widgets.error_label.set_visible(true);
                widgets.percent_label.set_text("!");
                widgets.volume_scale.set_value(0.0);
                widgets.volume_scale.set_sensitive(false);
                widgets.mute_button.set_active(false);
                set_mute_icon(&widgets.mute_button, false);
                widgets.mute_button.set_sensitive(false);
                widgets.output_selector.remove_all();
                widgets.input_selector.remove_all();
            }
        }

        self.syncing.set(false);
    }
}

fn mute_icon_name(muted: bool) -> &'static str {
    if muted {
        "audio-volume-muted-symbolic"
    } else {
        "audio-volume-high-symbolic"
    }
}

fn set_mute_icon(button: &gtk::ToggleButton, muted: bool) {
    if let Some(icon) = button.child().and_downcast::<gtk::Image>() {
        icon.set_icon_name(Some(mute_icon_name(muted)));
    }
}

fn connect_controls(
    widgets: &VolumeDropdownWidgets,
    sender: &ComponentSender<VolumeDropdown>,
    syncing: Rc<Cell<bool>>,
) {
    let input_sender = sender.input_sender().clone();
    let scale_syncing = syncing.clone();
    widgets.volume_scale.connect_value_changed(move |scale| {
        if !scale_syncing.get() {
            let _ = input_sender.send(VolumeDropdownInput::VolumeChanged(scale.value()));
        }
    });

    let input_sender = sender.input_sender().clone();
    let mute_syncing = syncing.clone();
    widgets.mute_button.connect_clicked(move |_| {
        if !mute_syncing.get() {
            let _ = input_sender.send(VolumeDropdownInput::ToggleMute);
        }
    });

    let input_sender = sender.input_sender().clone();
    let output_syncing = syncing.clone();
    widgets.output_selector.connect_changed(move |selector| {
        if output_syncing.get() {
            return;
        }

        let Some(key) = active_key(selector) else {
            return;
        };

        let _ = input_sender.send(VolumeDropdownInput::OutputSelected(key));
    });

    let input_sender = sender.input_sender().clone();
    widgets.input_selector.connect_changed(move |selector| {
        if syncing.get() {
            return;
        }

        let Some(key) = active_key(selector) else {
            return;
        };

        let _ = input_sender.send(VolumeDropdownInput::InputSelected(key));
    });
}

fn sync_selector(
    selector: &gtk::ComboBoxText,
    devices: &[AudioDeviceSummary],
    selected_key: Option<u32>,
) {
    selector.remove_all();

    for device in devices {
        selector.append(Some(&device.key.to_string()), &device.label);
    }

    if let Some(key) = selected_key {
        selector.set_active_id(Some(&key.to_string()));
    }

    selector.set_sensitive(!devices.is_empty());
}

fn active_key(selector: &gtk::ComboBoxText) -> Option<u32> {
    selector.active_id()?.parse().ok()
}
