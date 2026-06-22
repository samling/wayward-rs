use std::cell::Cell;
use std::rc::Rc;

use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

use crate::bar::{dropdown, layout::BarEdge, widget::BarRegion};

use super::sunsetr::SunsetrState;

#[derive(Clone, Debug)]
pub(super) struct BrightnessDropdownSnapshot {
    pub(super) percent: f64,
    pub(super) sunsetr_state: SunsetrState,
}

pub(super) struct BrightnessDropdown {
    edge: BarEdge,
    region: BarRegion,
    snapshot: Option<BrightnessDropdownSnapshot>,
    unavailable: Option<String>,
    syncing: Rc<Cell<bool>>,
}

pub(super) struct BrightnessDropdownInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
}

#[derive(Debug)]
pub(super) enum BrightnessDropdownInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetSnapshot(BrightnessDropdownSnapshot),
    SetUnavailable(String),
    BrightnessChanged(f64),
    SunsetrActionClicked,
}

#[derive(Debug)]
pub(super) enum BrightnessDropdownOutput {
    Opened,
    SetBrightness(f64),
    SetSunsetrPaused(bool),
}

#[relm4::component(pub(super))]
impl Component for BrightnessDropdown {
    type Init = BrightnessDropdownInit;
    type Input = BrightnessDropdownInput;
    type Output = BrightnessDropdownOutput;
    type CommandOutput = ();

    view! {
        #[root]
        #[name = "popover"]
        gtk::Popover {
            set_has_arrow: false,
            set_autohide: true,
            add_css_class: "dropdown",
            add_css_class: "brightness-dropdown",

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
                    add_css_class: "brightness-dropdown-content",
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    gtk::Box {
                        add_css_class: "dropdown-header",
                        add_css_class: "brightness-dropdown-header",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk::Label {
                            add_css_class: "dropdown-title",
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                            set_text: "Brightness",
                        },

                        #[name = "percent_label"]
                        gtk::Label {
                            add_css_class: "brightness-dropdown-percent",
                            set_halign: gtk::Align::End,
                        },
                    },

                    #[name = "error_label"]
                    gtk::Label {
                        add_css_class: "brightness-error",
                        set_halign: gtk::Align::Start,
                        set_visible: false,
                    },

                    gtk::Box {
                        add_css_class: "control-row",
                        add_css_class: "brightness-control-row",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,

                        #[name = "brightness_scale"]
                        gtk::Scale {
                            add_css_class: "control-scale",
                            set_hexpand: true,
                            set_halign: gtk::Align::Fill,
                            set_orientation: gtk::Orientation::Horizontal,
                            set_draw_value: false,
                            set_range: (0.0, 100.0),
                            set_increments: (1.0, 10.0),
                        },
                    },

                    gtk::Box {
                        add_css_class: "control-row",
                        add_css_class: "blue-light-row",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk::Label {
                            add_css_class: "control-label",
                            set_text: "Blue light filter",
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                        },

                        #[name = "sunsetr_status_label"]
                        gtk::Label {
                            add_css_class: "blue-light-status",
                            set_halign: gtk::Align::End,
                        },

                        #[name = "sunsetr_button"]
                        gtk::Button {
                            add_css_class: "control-toggle",
                            set_cursor_from_name: Some("pointer"),
                            set_label: "Pause",
                        },
                    },

                    #[name = "sunsetr_detail_label"]
                    gtk::Label {
                        add_css_class: "blue-light-detail",
                        set_halign: gtk::Align::Start,
                        set_wrap: true,
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
            unavailable: Some("Brightness has not loaded yet".to_string()),
            syncing: Rc::new(Cell::new(false)),
        };

        let widgets = view_output!();

        dropdown::connect_revealer(&widgets.popover, &widgets.revealer);
        connect_controls(&widgets, &sender, model.syncing.clone());
        connect_opened(&widgets.popover, &sender);

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
            BrightnessDropdownInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
            }
            BrightnessDropdownInput::SetSnapshot(snapshot) => {
                self.snapshot = Some(snapshot);
                self.unavailable = None;
                self.sync_widgets(widgets);
            }
            BrightnessDropdownInput::SetUnavailable(error) => {
                self.snapshot = None;
                self.unavailable = Some(error);
                self.sync_widgets(widgets);
            }
            BrightnessDropdownInput::BrightnessChanged(percent) => {
                let _ = sender.output(BrightnessDropdownOutput::SetBrightness(percent));
            }
            BrightnessDropdownInput::SunsetrActionClicked => {
                if let Some(paused) = self
                    .snapshot
                    .as_ref()
                    .and_then(|snapshot| snapshot.sunsetr_state.action_paused())
                {
                    let _ = sender.output(BrightnessDropdownOutput::SetSunsetrPaused(paused));
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl BrightnessDropdown {
    fn sync_widgets(&self, widgets: &BrightnessDropdownWidgets) {
        self.syncing.set(true);

        match &self.snapshot {
            Some(snapshot) => {
                widgets.error_label.set_visible(false);
                widgets.brightness_scale.set_sensitive(true);
                widgets.brightness_scale.set_value(snapshot.percent);
                widgets
                    .percent_label
                    .set_text(&format!("{:.0}%", snapshot.percent));
                widgets
                    .sunsetr_status_label
                    .set_text(snapshot.sunsetr_state.status_text());
                widgets
                    .sunsetr_detail_label
                    .set_text(&snapshot.sunsetr_state.detail_text());

                if let Some(label) = snapshot.sunsetr_state.action_label() {
                    widgets.sunsetr_button.set_label(label);
                    widgets.sunsetr_button.set_sensitive(true);
                    widgets
                        .sunsetr_button
                        .set_tooltip_text(Some(if label == "Pause" {
                            "Pause sunsetr on the daytime preset"
                        } else {
                            "Resume sunsetr automatic mode"
                        }));
                } else {
                    widgets.sunsetr_button.set_label("Unavailable");
                    widgets.sunsetr_button.set_sensitive(false);
                    widgets
                        .sunsetr_button
                        .set_tooltip_text(Some("sunsetr is not available"));
                }
            }
            None => {
                let error = self
                    .unavailable
                    .as_deref()
                    .unwrap_or("Brightness unavailable");
                widgets.error_label.set_text(error);
                widgets.error_label.set_visible(true);
                widgets.percent_label.set_text("!");
                widgets.brightness_scale.set_value(0.0);
                widgets.brightness_scale.set_sensitive(false);
                widgets.sunsetr_status_label.set_text("Unknown");
                widgets.sunsetr_detail_label.set_text("");
                widgets.sunsetr_button.set_sensitive(false);
            }
        }

        self.syncing.set(false);
    }
}

fn connect_controls(
    widgets: &BrightnessDropdownWidgets,
    sender: &ComponentSender<BrightnessDropdown>,
    syncing: Rc<Cell<bool>>,
) {
    let input_sender = sender.input_sender().clone();
    let scale_syncing = syncing.clone();
    widgets
        .brightness_scale
        .connect_value_changed(move |scale| {
            if !scale_syncing.get() {
                let _ =
                    input_sender.send(BrightnessDropdownInput::BrightnessChanged(scale.value()));
            }
        });

    let input_sender = sender.input_sender().clone();
    widgets.sunsetr_button.connect_clicked(move |_| {
        if syncing.get() {
            return;
        }

        let _ = input_sender.send(BrightnessDropdownInput::SunsetrActionClicked);
    });
}

fn connect_opened(popover: &gtk::Popover, sender: &ComponentSender<BrightnessDropdown>) {
    let output_sender = sender.output_sender().clone();
    popover.connect_map(move |_| {
        let _ = output_sender.send(BrightnessDropdownOutput::Opened);
    });
}
