use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;
use relm4::prelude::*;

use crate::workspace::WorkspaceSummary;

pub struct Bar {
    workspaces: Vec<WorkspaceSummary>,
    status: Option<String>,
}

#[derive(Debug)]
pub enum BarMsg {
    WorkspacesChanged(Vec<WorkspaceSummary>),
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
            set_resizable: false,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                add_css_class: "bar",
                set_spacing: 8,
                set_margin_start: 8,
                set_margin_end: 8,
                set_margin_top: 4,
                set_margin_bottom: 4,

                #[name = "workspace_row"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 4,
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

        let model = Bar {
            workspaces: Vec::new(),
            status: Some("Connecting to Niri".to_string()),
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

impl Bar {
    fn render_workspace_row(&self, row: &gtk::Box) {
        while let Some(child) = row.first_child() {
            row.remove(&child);
        }

        if let Some(status) = &self.status {
            let label = gtk::Label::new(Some(status));
            label.add_css_class("status");
            row.append(&label);
            return;
        }

        for workspace in &self.workspaces {
            let label = gtk::Label::new(Some(&workspace.label()));

            for class_name in workspace.css_classes() {
                label.add_css_class(class_name);
            }

            row.append(&label);
        }
    }
}
