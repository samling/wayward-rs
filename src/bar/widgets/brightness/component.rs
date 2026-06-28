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
use super::sunsetr::{PendingSunsetrAction, SunsetrState};

pub(super) struct BrightnessComponent {
    edge: BarEdge,
    region: BarRegion,
    snapshot: Option<BrightnessSnapshot>,
    unavailable: Option<String>,
    dropdown: Controller<BrightnessDropdown>,
    bar_sender: relm4::Sender<BarMsg>,
    config: BrightnessConfig,
    sunsetr_state: SunsetrState,
    pending_sunsetr_action: Option<PendingSunsetrAction>,
}

#[derive(Debug)]
pub(super) enum BrightnessInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetSnapshot(BrightnessSnapshot),
    SetUnavailable(String),
    SetBrightness(f64),
    SetSunsetrPaused(bool),
    CheckSunsetrState,
    SetSunsetrState(SunsetrState),
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
            #[name = "content"]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 3,

                gtk::Image {
                    add_css_class: "brightness-icon",
                    set_icon_name: Some("display-brightness-symbolic"),
                },

                #[name = "percent"]
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
                BrightnessDropdownOutput::Opened => BrightnessInput::CheckSunsetrState,
                BrightnessDropdownOutput::SetBrightness(percent) => {
                    BrightnessInput::SetBrightness(percent)
                }
                BrightnessDropdownOutput::SetSunsetrPaused(paused) => {
                    BrightnessInput::SetSunsetrPaused(paused)
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
            sunsetr_state: SunsetrState::Unknown("Checking sunsetr".to_string()),
            pending_sunsetr_action: None,
        };

        let widgets = view_output!();
        crate::bar::style::add_bar_item_content_classes(&widgets.content, "brightness-content");
        crate::bar::style::configure_bar_label(&widgets.percent);

        root.set_popover(Some(model.dropdown.widget()));

        model.check_sunsetr_state(sender.input_sender().clone());

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
            BrightnessInput::SetSunsetrPaused(paused) => {
                let preset = if paused {
                    self.config.sunsetr.paused_preset.clone()
                } else {
                    self.config.sunsetr.automatic_preset.clone()
                };

                self.pending_sunsetr_action = Some(PendingSunsetrAction::new(paused));
                self.sunsetr_state = self
                    .sunsetr_state
                    .with_action_applied(paused, &self.config.sunsetr);
                self.sync_dropdown_snapshot();
                self.send_action(WidgetAction::Brightness(
                    BrightnessAction::SetSunsetrPreset { preset },
                ));
                self.check_sunsetr_state_delayed(_sender.input_sender().clone());
            }
            BrightnessInput::CheckSunsetrState => {
                self.check_sunsetr_state(_sender.input_sender().clone());
            }
            BrightnessInput::SetSunsetrState(state) => {
                if let Some(mut pending) = self.pending_sunsetr_action {
                    if !pending.accepts_status(&state) {
                        self.pending_sunsetr_action = Some(pending);
                        self.check_sunsetr_state_delayed(_sender.input_sender().clone());
                        return;
                    }

                    self.pending_sunsetr_action = None;
                }

                self.sunsetr_state = state;
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
            sunsetr_state: self.sunsetr_state.clone(),
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

    fn check_sunsetr_state(&self, input_sender: relm4::Sender<BrightnessInput>) {
        let config = self.config.sunsetr.clone();

        relm4::spawn(async move {
            let state = super::sunsetr::current_state(config).await;
            let _ = input_sender.send(BrightnessInput::SetSunsetrState(state));
        });
    }

    fn check_sunsetr_state_delayed(&self, input_sender: relm4::Sender<BrightnessInput>) {
        let config = self.config.sunsetr.clone();

        relm4::spawn(async move {
            relm4::tokio::time::sleep(Duration::from_millis(300)).await;
            let state = super::sunsetr::current_state(config).await;
            let _ = input_sender.send(BrightnessInput::SetSunsetrState(state));
        });
    }
}
