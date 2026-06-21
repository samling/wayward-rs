use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentController, Controller};

use crate::bar::BarMsg;
use crate::bar::layout::BarEdge;
use crate::bar::widget::{BarRegion, UpdatesAction, WidgetAction, WidgetEvent};
use crate::bar::widgets::updates::service::UpdatesServiceConfig;

use super::dropdown::{
    UpdatesDropdown, UpdatesDropdownInit, UpdatesDropdownInput, UpdatesDropdownOutput,
};
use super::model::UpdatesSnapshot;

pub(super) struct UpdatesComponent {
    edge: BarEdge,
    region: BarRegion,
    snapshot: Option<UpdatesSnapshot>,
    unavailable: Option<String>,
    dropdown: Controller<UpdatesDropdown>,
    bar_sender: relm4::Sender<BarMsg>,
    config: UpdatesServiceConfig,
}

#[derive(Debug)]
pub(super) enum UpdatesInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetSnapshot(UpdatesSnapshot),
    SetUnavailable(String),
    RefreshRequested,
}

pub(super) struct UpdatesInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
    pub(super) bar_sender: relm4::Sender<BarMsg>,
    pub(super) config: UpdatesServiceConfig,
}

#[relm4::component(pub(super))]
impl SimpleComponent for UpdatesComponent {
    type Init = UpdatesInit;
    type Input = UpdatesInput;
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
                add_css_class: "updates-content",
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 4,

                gtk::Image {
                    add_css_class: "updates-icon",
                    set_icon_name: Some("software-update-available-symbolic"),

                    #[watch]
                    set_visible: !model.is_refreshing(),
                },

                gtk::Spinner {
                    add_css_class: "updates-spinner",

                    #[watch]
                    set_visible: model.is_refreshing(),

                    #[watch]
                    set_spinning: model.is_refreshing(),
                },

                #[name = "count"]
                gtk::Label {
                    add_css_class: "updates-count",

                    #[watch]
                    set_text: &model.count_text(),
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let dropdown = UpdatesDropdown::builder()
            .launch(UpdatesDropdownInit {
                edge: init.edge,
                region: init.region,
            })
            .forward(sender.input_sender(), |output| match output {
                UpdatesDropdownOutput::RefreshRequested => UpdatesInput::RefreshRequested,
            });

        let model = Self {
            edge: init.edge,
            region: init.region,
            snapshot: None,
            unavailable: Some("Updates have not loaded yet".to_string()),
            dropdown,
            bar_sender: init.bar_sender,
            config: init.config,
        };

        let _updates_watcher =
            super::service::start(model.bar_sender.clone(), model.config.clone());

        let widgets = view_output!();
        crate::bar::style::configure_bar_item_content(&widgets.content);
        crate::bar::style::configure_bar_label(&widgets.count);

        root.set_popover(Some(model.dropdown.widget()));

        let bar_sender = model.bar_sender.clone();
        root.connect_notify_local(Some("active"), move |button, _| {
            if button.is_active() {
                send_refresh_request(&bar_sender);
            }
        });

        let right_click = gtk::GestureClick::new();
        right_click.set_button(gtk::gdk::BUTTON_SECONDARY);

        let bar_sender = model.bar_sender.clone();
        right_click.connect_released(move |_, _, _, _| {
            send_refresh_request(&bar_sender);
        });

        root.add_controller(right_click);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            UpdatesInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
                self.dropdown
                    .emit(UpdatesDropdownInput::SetPlacement { edge, region });
            }
            UpdatesInput::SetSnapshot(snapshot) => {
                self.dropdown
                    .emit(UpdatesDropdownInput::SetSnapshot(snapshot.clone()));
                self.snapshot = Some(snapshot);
                self.unavailable = None;
            }
            UpdatesInput::SetUnavailable(error) => {
                self.dropdown
                    .emit(UpdatesDropdownInput::SetUnavailable(error.clone()));
                self.snapshot = None;
                self.unavailable = Some(error);
            }
            UpdatesInput::RefreshRequested => {
                send_refresh_request(&self.bar_sender);
            }
        }
    }
}

impl UpdatesComponent {
    fn count_text(&self) -> String {
        self.snapshot
            .as_ref()
            .map(|snapshot| snapshot.packages.len().to_string())
            .unwrap_or_else(|| "!".to_string())
    }

    fn tooltip_text(&self) -> Option<String> {
        if let Some(error) = &self.unavailable {
            Some(error.clone())
        } else {
            self.snapshot
                .as_ref()
                .map(|snapshot| format!("{} update(s)", snapshot.packages.len()))
        }
    }

    fn root_css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["bar-item", "updates", "flat"];

        if self.is_refreshing() {
            classes.push("refreshing");
        }

        classes
    }

    fn is_refreshing(&self) -> bool {
        self.snapshot
            .as_ref()
            .map(|snapshot| snapshot.refreshing)
            .unwrap_or(false)
    }
}

fn send_refresh_request(bar_sender: &relm4::Sender<BarMsg>) {
    let _ = bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
        widget_id: "updates",
        action: WidgetAction::Updates(UpdatesAction::Refresh),
    }));
}
