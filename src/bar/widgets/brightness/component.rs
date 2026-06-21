use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentController, Controller};
use std::time::Duration;

use crate::bar::BarMsg;
use crate::bar::layout::BarEdge;
use crate::bar::state::BrightnessSnapshot;
use crate::bar::widget::{BarRegion, BrightnessAction, WidgetAction, WidgetEvent};

use super::config::BrightnessConfig;
use super::dropdown::{
    BrightnessDropdown, BrightnessDropdownInit, BrightnessDropdownInput, BrightnessDropdownOutput,
    BrightnessDropdownSnapshot,
};

pub(super) struct BrightnessComponent {
    edge: BarEdge,
    region: BarRegion,
    snapshot: Option<BrightnessSnapshot>,
    unavailable: Option<String>,
    dropdown: Controller<BrightnessDropdown>,
    bar_sender: relm4::Sender<BarMsg>,
    config: BrightnessConfig,
    blue_light_enabled: Option<bool>,
}

#[derive(Debug)]
pub(super) enum BrightnessInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetSnapshot(BrightnessSnapshot),
    SetUnavailable(String),
    SetBrightness(f64),
    SetBlueLightEnabled(bool),
    CheckBlueLightState,
    SetBlueLightState(Option<bool>),
}

pub(super) struct BrightnessInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
    pub(super) bar_sender: relm4::Sender<BarMsg>,
    pub(super) config: BrightnessConfig,
}

#[relm4::component(pub(super))]
impl SimpleComponent for BrightnessComponent {
    type Init = BrightnessInit;
    type Input = BrightnessInput;
    type Output = ();

    view! {
        gtk::MenuButton {
            set_always_show_arrow: false,
            set_cursor_from_name: Some("pointer"),
            add_css_class: "bar-item",
            add_css_class: "brightness",
            add_css_class: "flat",

            #[watch]
            set_tooltip_text: model.tooltip_text().as_deref(),

            #[wrap(Some)]
            set_child = &gtk::Box {
                add_css_class: "bar-item-content",
                add_css_class: "brightness-content",
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 3,

                gtk::Image {
                    add_css_class: "brightness-icon",
                    set_icon_name: Some("display-brightness-symbolic"),
                },

                gtk::Label {
                    add_css_class: "brightness-percent",

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
        let dropdown = BrightnessDropdown::builder()
            .launch(BrightnessDropdownInit {
                edge: init.edge,
                region: init.region,
            })
            .forward(sender.input_sender(), |output| match output {
                BrightnessDropdownOutput::Opened => BrightnessInput::CheckBlueLightState,
                BrightnessDropdownOutput::SetBrightness(percent) => {
                    BrightnessInput::SetBrightness(percent)
                }
                BrightnessDropdownOutput::SetBlueLightEnabled(enabled) => {
                    BrightnessInput::SetBlueLightEnabled(enabled)
                }
            });

        let model = Self {
            edge: init.edge,
            region: init.region,
            snapshot: None,
            unavailable: Some("Brightness has not loaded yet".to_string()),
            dropdown,
            bar_sender: init.bar_sender,
            config: init.config,
            blue_light_enabled: None,
        };

        let widgets = view_output!();

        root.set_popover(Some(model.dropdown.widget()));

        model.check_blue_light_state(sender.input_sender().clone());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            BrightnessInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
                self.dropdown
                    .emit(BrightnessDropdownInput::SetPlacement { edge, region });
            }
            BrightnessInput::SetSnapshot(snapshot) => {
                self.dropdown.emit(BrightnessDropdownInput::SetSnapshot(
                    self.dropdown_snapshot(&snapshot),
                ));
                self.snapshot = Some(snapshot);
                self.unavailable = None;
            }
            BrightnessInput::SetUnavailable(error) => {
                self.dropdown
                    .emit(BrightnessDropdownInput::SetUnavailable(error.clone()));
                self.snapshot = None;
                self.unavailable = Some(error);
            }
            BrightnessInput::SetBrightness(percent) => {
                self.send_action(WidgetAction::Brightness(BrightnessAction::SetBrightness {
                    percent,
                }));
            }
            BrightnessInput::SetBlueLightEnabled(enabled) => {
                if let Some(command) = self.config.blue_light_command_for_state(enabled) {
                    self.blue_light_enabled = Some(enabled);
                    self.sync_dropdown_snapshot();
                    self.send_action(WidgetAction::Brightness(
                        BrightnessAction::RunBlueLightCommand {
                            command: command.to_string(),
                        },
                    ));
                    self.check_blue_light_state_delayed(_sender.input_sender().clone());
                }
            }
            BrightnessInput::CheckBlueLightState => {
                self.check_blue_light_state(_sender.input_sender().clone());
            }
            BrightnessInput::SetBlueLightState(enabled) => {
                self.blue_light_enabled = enabled;
                self.sync_dropdown_snapshot();
            }
        }
    }
}

impl BrightnessComponent {
    fn percent_text(&self) -> String {
        self.snapshot
            .as_ref()
            .map(|snapshot| format!("{:.0}%", snapshot.percent))
            .unwrap_or_else(|| "!".to_string())
    }

    fn tooltip_text(&self) -> Option<String> {
        if let Some(error) = &self.unavailable {
            Some(error.clone())
        } else {
            self.snapshot
                .as_ref()
                .map(|snapshot| format!("Brightness {:.0}%", snapshot.percent))
        }
    }

    fn dropdown_snapshot(&self, snapshot: &BrightnessSnapshot) -> BrightnessDropdownSnapshot {
        BrightnessDropdownSnapshot {
            percent: snapshot.percent,
            blue_light_configured: self.config.blue_light_toggle_configured(),
            blue_light_enabled: self.blue_light_enabled,
        }
    }

    fn sync_dropdown_snapshot(&self) {
        if let Some(snapshot) = &self.snapshot {
            self.dropdown.emit(BrightnessDropdownInput::SetSnapshot(
                self.dropdown_snapshot(snapshot),
            ));
        }
    }

    fn send_action(&self, action: WidgetAction) {
        let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
            widget_id: "brightness",
            action,
        }));
    }

    fn check_blue_light_state(&self, input_sender: relm4::Sender<BrightnessInput>) {
        if !self.config.blue_light_toggle_configured() {
            let _ = input_sender.send(BrightnessInput::SetBlueLightState(None));
            return;
        }

        let command = self.config.blue_light_state_command.clone();

        relm4::spawn(async move {
            let enabled = super::service::blue_light_enabled(&command).await;
            let _ = input_sender.send(BrightnessInput::SetBlueLightState(enabled));
        });
    }

    fn check_blue_light_state_delayed(&self, input_sender: relm4::Sender<BrightnessInput>) {
        if !self.config.blue_light_toggle_configured() {
            return;
        }

        let command = self.config.blue_light_state_command.clone();

        relm4::spawn(async move {
            relm4::tokio::time::sleep(Duration::from_millis(300)).await;
            let enabled = super::service::blue_light_enabled(&command).await;
            let _ = input_sender.send(BrightnessInput::SetBlueLightState(enabled));
        });
    }
}
