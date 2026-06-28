use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
};

use crate::bar::BarMsg;
use crate::bar::layout::BarEdge;
use crate::bar::menu_button;
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

pub(super) struct ActionMenuWidgets;

impl SimpleComponent for ActionMenuComponent {
    type Init = ActionMenuInit;
    type Input = ActionMenuInput;
    type Output = ();
    type Root = gtk::MenuButton;
    type Widgets = ActionMenuWidgets;

    fn init_root() -> Self::Root {
        gtk::MenuButton::new()
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

        let content =
            menu_button::content_box(gtk::Orientation::Horizontal, 0, "action-menu-content");

        let bar_icon = gtk::Label::new(Some("\u{f303}"));
        bar_icon.add_css_class("action-menu-bar-icon");
        crate::bar::style::configure_bar_label(&bar_icon);
        content.append(&bar_icon);

        menu_button::configure_root(&root, "action-menu", None);
        menu_button::attach_content(&root, &content);
        menu_button::attach_popover(&root, model.dropdown.widget());

        ComponentParts {
            model,
            widgets: ActionMenuWidgets,
        }
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
