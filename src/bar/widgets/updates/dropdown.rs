use relm4::factory::FactoryVecDeque;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

use crate::bar::{dropdown, layout::BarEdge, widget::BarRegion};

use super::model::{UpdatePackage, UpdatesSnapshot};
use super::row::UpdateRow;

pub(super) struct UpdatesDropdown {
    edge: BarEdge,
    region: BarRegion,
    packages: Vec<UpdatePackage>,
    last_error: Option<String>,
    refreshing: bool,
    rows: FactoryVecDeque<UpdateRow>,
}

pub(super) struct UpdatesDropdownInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
}

#[derive(Debug)]
pub(super) enum UpdatesDropdownInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetSnapshot(UpdatesSnapshot),
    SetUnavailable(String),
}

#[relm4::component(pub(super))]
impl SimpleComponent for UpdatesDropdown {
    type Init = UpdatesDropdownInit;
    type Input = UpdatesDropdownInput;
    type Output = ();

    view! {
        #[root]
        #[name = "popover"]
        gtk::Popover {
            set_has_arrow: false,
            set_autohide: true,
            add_css_class: "dropdown",
            add_css_class: "updates-dropdown",

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
                    add_css_class: "updates-dropdown-content",
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,

                    gtk::Box {
                        add_css_class: "updates-dropdown-header",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                        set_hexpand: true,

                        gtk::Label {
                            add_css_class: "dropdown-title",
                            add_css_class: "updates-dropdown-title",
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                            set_text: "Updates",
                        },

                        gtk::Label {
                            add_css_class: "updates-refreshing",
                            set_halign: gtk::Align::End,

                            #[watch]
                            set_visible: model.refreshing,

                            set_text: "Refreshing",
                        },
                    },

                    gtk::Label {
                        add_css_class: "updates-error",
                        set_halign: gtk::Align::Start,

                        #[watch]
                        set_visible: model.last_error.is_some(),

                        #[watch]
                        set_text: model.last_error.as_deref().unwrap_or(""),
                    },

                    gtk::Label {
                        add_css_class: "updates-empty",
                        set_halign: gtk::Align::Start,
                        set_text: "System is up to date",

                        #[watch]
                        set_visible: model.packages.is_empty() && model.last_error.is_none(),
                    },

                    #[name = "scroller"]
                    gtk::ScrolledWindow {
                        add_css_class: "updates-list-scroll",
                        set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                        set_min_content_width: 380,
                        set_propagate_natural_height: true,
                        set_max_content_height: 700,

                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            add_css_class: "updates-list",
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 6,

                            #[local_ref]
                            list -> gtk::ListBox {
                                add_css_class: "updates-list-items",
                                set_selection_mode: gtk::SelectionMode::None,
                            }
                        },
                    },
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list = gtk::ListBox::default();
        let rows = FactoryVecDeque::builder()
            .launch(list.clone())
            .detach();

        let model = Self {
            edge: init.edge,
            region: init.region,
            packages: Vec::new(),
            last_error: None,
            refreshing: false,
            rows,
        };

        let widgets = view_output!();

        dropdown::connect_revealer(&widgets.popover, &widgets.revealer);

        let adjustment = widgets.scroller.vadjustment();
        adjustment.set_step_increment(72.0);
        adjustment.set_page_increment(240.0);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            UpdatesDropdownInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
            }
            UpdatesDropdownInput::SetSnapshot(snapshot) => {
                self.packages = snapshot.packages.clone();
                self.last_error = snapshot.last_error;
                self.refreshing = snapshot.refreshing;
                self.sync_rows(snapshot.packages);
            }
            UpdatesDropdownInput::SetUnavailable(error) => {
                self.packages.clear();
                self.last_error = Some(error);
                self.refreshing = false;
                self.rows.guard().clear();
            }
        }
    }
}

impl UpdatesDropdown {
    fn sync_rows(&mut self, packages: Vec<UpdatePackage>) {
        let mut rows = self.rows.guard();

        for index in (0..rows.len()).rev() {
            if !packages.iter().any(|package| package.name == rows[index].name()) {
                rows.remove(index);
            }
        }

        for (target_index, package) in packages.iter().enumerate() {
            if target_index < rows.len() && rows[target_index].name() == package.name {
                if let Some(row) = rows.get_mut(target_index) {
                    row.set_package(package.clone());
                }
                continue;
            }

            let existing_index = rows.iter().position(|row| row.name() == package.name);

            if let Some(existing_index) = existing_index {
                rows.move_to(existing_index, target_index);
                if let Some(row) = rows.get_mut(target_index) {
                    row.set_package(package.clone());
                }
            } else {
                rows.insert(target_index, package.clone());
            }
        }

        while rows.len() > packages.len() {
            rows.pop_back();
        }
    }
}