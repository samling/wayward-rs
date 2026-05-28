mod clock;
mod workspaces;

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;
use relm4::prelude::*;

use crate::workspace::WorkspaceSummary;

pub struct Bar {
    pub(super) workspaces: Vec<WorkspaceSummary>,
    pub(super) status: Option<String>,
    pub(super) clock_text: String,
}

#[derive(Debug)]
pub enum BarMsg {
    WorkspacesChanged(Vec<WorkspaceSummary>),
    ClockChanged(String),
    NiriUnavailable(String),
    UpdatesStopped,
}

#[relm4::component(pub)]
impl SimpleComponent for Bar {
    type Init = ();
    type Input = BarMsg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Wayward"),
            set_default_height: 32,
            set_resizable: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                add_css_class: "bar",

                #[name = "left_region"]
                gtk::Box {
                    add_css_class: "bar-region",

                    #[name = "workspace_row"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 4,
                    }
                },

                #[name = "center_region"]
                gtk::Box {
                    set_hexpand: true,
                    add_css_class: "bar-region",
                },

                #[name = "right_region"]
                gtk::Box {
                    add_css_class: "bar-region",

                    #[name = "clock_label"]
                    gtk::Label {
                        add_css_class: "bar-item",
                        add_css_class: "clock",
                        #[watch]
                        set_label: &model.clock_text
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_layer(Layer::Top);
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);
        root.auto_exclusive_zone_enable();
        root.set_keyboard_mode(KeyboardMode::None);
        root.set_namespace(Some("wayward"));

        let clock_sender = sender.input_sender().clone();
        relm4::spawn(async move {
            clock::run_clock(clock_sender).await;
        });

        let model = Bar {
            workspaces: Vec::new(),
            status: Some("Connecting to Niri".to_string()),
            clock_text: clock::current_time_text(),
        };
        let widgets = view_output!();
        model.render_workspace_row(&widgets.workspace_row);

        let input_sender = sender.input_sender().clone();
        relm4::spawn(async move {
            crate::niri::run_workspace_watcher(input_sender).await;
        });

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        _sender: ComponentSender<Self>,
    ) {
        match message {
            BarMsg::WorkspacesChanged(workspaces) => {
                self.workspaces = workspaces;
                self.status = None;
            }
            BarMsg::ClockChanged(clock_text) => {
                self.clock_text = clock_text;
            }
            BarMsg::NiriUnavailable(error) => {
                self.workspaces.clear();
                self.status = Some(format!("Niri unavailable: {error}"));
            }
            BarMsg::UpdatesStopped => {
                self.status = Some("Niri updates stopped".to_string());
            }
        }
    }

    fn pre_view() {
        self.render_workspace_row(&workspace_row);
    }
}
