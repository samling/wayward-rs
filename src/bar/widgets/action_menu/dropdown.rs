use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

use crate::bar::widget::{ActionMenuAction, WidgetAction, WidgetEvent};
use crate::bar::{BarMsg, dropdown, layout::BarEdge, widget::BarRegion};

pub(super) struct ActionMenuDropdown {
    edge: BarEdge,
    region: BarRegion,
    bar_sender: relm4::Sender<BarMsg>,
}

pub(super) struct ActionMenuDropdownInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
    pub(super) bar_sender: relm4::Sender<BarMsg>,
}

#[derive(Debug)]
pub(super) enum ActionMenuDropdownInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    Run(ActionMenuAction),
}

#[relm4::component(pub(super))]
impl SimpleComponent for ActionMenuDropdown {
    type Init = ActionMenuDropdownInit;
    type Input = ActionMenuDropdownInput;
    type Output = ();

    view! {
        #[root]
        #[name = "popover"]
        gtk::Popover {
            set_has_arrow: false,
            set_autohide: true,
            add_css_class: "dropdown",
            add_css_class: "action-menu-dropdown",

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
                    add_css_class: "action-menu-dropdown-content",
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,

                    gtk::Box {
                        add_css_class: "action-menu-header",
                        set_orientation: gtk::Orientation::Horizontal,

                        gtk::Box {
                            set_hexpand: true,
                        },

                        gtk::Button {
                            add_css_class: "action-menu-power",
                            add_css_class: "flat",
                            set_width_request: 34,
                            set_height_request: 34,
                            set_cursor_from_name: Some("pointer"),
                            set_tooltip_text: Some("Power menu"),

                            gtk::Label {
                                add_css_class: "action-menu-action-icon",
                                add_css_class: "power",
                                set_text: "\u{f011}",
                            },

                            connect_clicked[sender] => move |_| {
                                sender.input(ActionMenuDropdownInput::Run(
                                    ActionMenuAction::PowerMenu,
                                ));
                            }
                        },
                    },

                    gtk::Label {
                        add_css_class: "action-menu-section-title",
                        set_halign: gtk::Align::Start,
                        set_text: "Screenshot",
                    },

                    gtk::Box {
                        add_css_class: "action-menu-screenshot-actions",
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                        set_homogeneous: true,

                        gtk::Button {
                            add_css_class: "action-menu-screenshot-action",
                            add_css_class: "flat",
                            set_cursor_from_name: Some("pointer"),
                            set_tooltip_text: Some("Screenshot region"),

                            gtk::Box {
                                add_css_class: "action-menu-button-content",
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 4,

                                gtk::Label {
                                    add_css_class: "action-menu-action-icon",
                                    set_text: "\u{f125}",
                                },

                                gtk::Label {
                                    add_css_class: "action-menu-action-label",
                                    set_text: "Region",
                                },
                            },

                            connect_clicked[sender] => move |_| {
                                sender.input(ActionMenuDropdownInput::Run(
                                    ActionMenuAction::ScreenshotRegion,
                                ));
                            }
                        },

                        gtk::Button {
                            add_css_class: "action-menu-screenshot-action",
                            add_css_class: "flat",
                            set_cursor_from_name: Some("pointer"),
                            set_tooltip_text: Some("Screenshot window"),

                            gtk::Box {
                                add_css_class: "action-menu-button-content",
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 4,

                                gtk::Label {
                                    add_css_class: "action-menu-action-icon",
                                    set_text: "\u{f2d0}",
                                },

                                gtk::Label {
                                    add_css_class: "action-menu-action-label",
                                    set_text: "Window",
                                },
                            },

                            connect_clicked[sender] => move |_| {
                                sender.input(ActionMenuDropdownInput::Run(
                                    ActionMenuAction::ScreenshotWindow,
                                ));
                            }
                        },

                        gtk::Button {
                            add_css_class: "action-menu-screenshot-action",
                            add_css_class: "flat",
                            set_cursor_from_name: Some("pointer"),
                            set_tooltip_text: Some("Screenshot screen"),

                            gtk::Box {
                                add_css_class: "action-menu-button-content",
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 4,

                                gtk::Label {
                                    add_css_class: "action-menu-action-icon",
                                    set_text: "\u{f108}",
                                },

                                gtk::Label {
                                    add_css_class: "action-menu-action-label",
                                    set_text: "Screen",
                                },
                            },

                            connect_clicked[sender] => move |_| {
                                sender.input(ActionMenuDropdownInput::Run(
                                    ActionMenuAction::ScreenshotScreen,
                                ));
                            }
                        },
                    },
                }
            },
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
            bar_sender: init.bar_sender,
        };

        let widgets = view_output!();

        dropdown::connect_revealer(&widgets.popover, &widgets.revealer);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ActionMenuDropdownInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
            }
            ActionMenuDropdownInput::Run(action) => {
                let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
                    widget_id: "action_menu",
                    action: WidgetAction::RunActionMenuAction { action },
                }));
            }
        }
    }
}