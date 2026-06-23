use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentController, Controller};

use crate::bar::BarMsg;
use crate::bar::layout::BarEdge;
use crate::bar::widget::BarRegion;

use super::config::ActionMenuConfig;
use super::dropdown::{ActionMenuDropdown, ActionMenuDropdownInit, ActionMenuDropdownInput};

pub(super) struct ActionMenuComponent {
    edge: BarEdge,
    region: BarRegion,
    dropdown: Controller<ActionMenuDropdown>,
}

#[derive(Debug)]
pub(super) enum ActionMenuInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
}

pub(super) struct ActionMenuInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
    pub(super) bar_sender: relm4::Sender<BarMsg>,
    pub(super) config: ActionMenuConfig,
}

#[relm4::component(pub(super))]
impl SimpleComponent for ActionMenuComponent {
    type Init = ActionMenuInit;
    type Input = ActionMenuInput;
    type Output = ();

    view! {
        gtk::MenuButton {
            set_always_show_arrow: false,
            set_cursor_from_name: Some("pointer"),
            add_css_class: "bar-item",
            add_css_class: "action-menu",
            add_css_class: "flat",

            #[wrap(Some)]
            #[name = "content"]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                #[name = "bar_icon"]
                gtk::Image {
                    add_css_class: "action-menu-bar-icon",
                    set_icon_name: Some("arch-linux-svgrepo-com"),
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let dropdown = ActionMenuDropdown::builder()
            .launch(ActionMenuDropdownInit {
                edge: init.edge,
                region: init.region,
                bar_sender: init.bar_sender,
                config: init.config.clone(),
            })
            .detach();

        let model = Self {
            edge: init.edge,
            region: init.region,
            dropdown,
        };

        let widgets = view_output!();
        crate::bar::style::add_bar_item_content_classes(&widgets.content, "action-menu-content");

        root.set_popover(Some(model.dropdown.widget()));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ActionMenuInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
                self.dropdown
                    .emit(ActionMenuDropdownInput::SetPlacement { edge, region });
            }
        }
    }
}
