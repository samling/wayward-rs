use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentController, Controller};

use crate::bar::BarMsg;
use crate::bar::layout::BarEdge;
use crate::bar::widget::{BarRegion, VolumeAction, WidgetAction, WidgetEvent};

use super::dropdown::{
    VolumeDropdown, VolumeDropdownInit, VolumeDropdownInput, VolumeDropdownOutput,
};
use super::model::VolumeSnapshot;

pub(super) struct VolumeComponent {
    edge: BarEdge,
    region: BarRegion,
    snapshot: Option<VolumeSnapshot>,
    unavailable: Option<String>,
    dropdown: Controller<VolumeDropdown>,
    bar_sender: relm4::Sender<BarMsg>,
}

#[derive(Debug)]
pub(super) enum VolumeInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetSnapshot(VolumeSnapshot),
    SetUnavailable(String),
    SetOutputVolume(f64),
    ToggleOutputMute,
    SetDefaultOutput(u32),
    SetDefaultInput(u32),
}

pub(super) struct VolumeInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
    pub(super) bar_sender: relm4::Sender<BarMsg>,
}

#[relm4::component(pub(super))]
impl SimpleComponent for VolumeComponent {
    type Init = VolumeInit;
    type Input = VolumeInput;
    type Output = ();

    view! {
        gtk::MenuButton {
            set_always_show_arrow: false,
            set_cursor_from_name: Some("pointer"),

            #[watch]
            set_css_classes: &model.root_css_classes(),

            #[watch]
            set_tooltip_text: model.tooltip_text().as_deref(),

            #[wrap(Some)]
            #[name = "content"]
            set_child = &gtk::Box {
                add_css_class: "bar-item-content",
                add_css_class: "volume-content",
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 3,

                gtk::Image {
                    add_css_class: "volume-icon",

                    #[watch]
                    set_icon_name: Some(model.icon_name()),
                },

                #[name = "percent"]
                gtk::Label {
                    add_css_class: "volume-percent",
                    set_width_chars: 4,
                    set_xalign: 1.0,

                    #[watch]
                    set_text: &model.percent_text(),
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let dropdown = VolumeDropdown::builder()
            .launch(VolumeDropdownInit {
                edge: init.edge,
                region: init.region,
            })
            .forward(sender.input_sender(), |output| match output {
                VolumeDropdownOutput::SetOutputVolume(percent) => {
                    VolumeInput::SetOutputVolume(percent)
                }
                VolumeDropdownOutput::ToggleOutputMute => VolumeInput::ToggleOutputMute,
                VolumeDropdownOutput::SetDefaultOutput(key) => VolumeInput::SetDefaultOutput(key),
                VolumeDropdownOutput::SetDefaultInput(key) => VolumeInput::SetDefaultInput(key),
            });

        let model = Self {
            edge: init.edge,
            region: init.region,
            snapshot: None,
            unavailable: Some("Volume has not loaded yet".to_string()),
            dropdown,
            bar_sender: init.bar_sender,
        };

        let widgets = view_output!();
        crate::bar::style::configure_bar_item_content(&widgets.content);
        crate::bar::style::configure_bar_label(&widgets.percent);

        root.set_popover(Some(model.dropdown.widget()));
        connect_right_click(&root, model.bar_sender.clone());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            VolumeInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
                self.dropdown
                    .emit(VolumeDropdownInput::SetPlacement { edge, region });
            }
            VolumeInput::SetSnapshot(snapshot) => {
                self.dropdown
                    .emit(VolumeDropdownInput::SetSnapshot(snapshot.clone()));
                self.snapshot = Some(snapshot);
                self.unavailable = None;
            }
            VolumeInput::SetUnavailable(error) => {
                self.dropdown
                    .emit(VolumeDropdownInput::SetUnavailable(error.clone()));
                self.snapshot = None;
                self.unavailable = Some(error);
            }
            VolumeInput::SetOutputVolume(percent) => {
                self.send_action(WidgetAction::Volume(VolumeAction::SetOutputVolume {
                    percent,
                }));
            }
            VolumeInput::ToggleOutputMute => {
                self.send_action(WidgetAction::Volume(VolumeAction::ToggleOutputMute));
            }
            VolumeInput::SetDefaultOutput(key) => {
                self.send_action(WidgetAction::Volume(VolumeAction::SetDefaultOutput { key }));
            }
            VolumeInput::SetDefaultInput(key) => {
                self.send_action(WidgetAction::Volume(VolumeAction::SetDefaultInput { key }));
            }
        }
    }
}

impl VolumeComponent {
    fn icon_name(&self) -> &'static str {
        self.snapshot
            .as_ref()
            .map(VolumeSnapshot::icon_name)
            .unwrap_or("audio-volume-muted-symbolic")
    }

    fn percent_text(&self) -> String {
        self.snapshot
            .as_ref()
            .map(VolumeSnapshot::percent_text)
            .unwrap_or_else(|| "!".to_string())
    }

    fn tooltip_text(&self) -> Option<String> {
        if let Some(error) = &self.unavailable {
            Some(error.clone())
        } else {
            self.snapshot
                .as_ref()
                .map(|snapshot| format!("Volume {}", snapshot.percent_text()))
        }
    }

    fn root_css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["bar-item", "volume", "flat"];

        if self
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.muted)
            .unwrap_or(false)
        {
            classes.push("muted");
        }

        classes
    }

    fn send_action(&self, action: WidgetAction) {
        let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
            widget_id: "volume",
            action,
        }));
    }
}

fn connect_right_click(root: &gtk::MenuButton, bar_sender: relm4::Sender<BarMsg>) {
    let right_click = gtk::GestureClick::new();
    right_click.set_button(gtk::gdk::BUTTON_SECONDARY);

    right_click.connect_released(move |_, _, _, _| {
        let _ = bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
            widget_id: "volume",
            action: WidgetAction::Volume(VolumeAction::ToggleOutputMute),
        }));
    });

    root.add_controller(right_click);
}
