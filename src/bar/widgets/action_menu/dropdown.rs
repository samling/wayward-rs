use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

use crate::bar::widget::{ActionMenuAction, ActionMenuCommand, WidgetAction, WidgetEvent};
use crate::bar::{BarMsg, dropdown, layout::BarEdge, widget::BarRegion};

use super::config::{
    ActionMenuActionConfig, ActionMenuActionKind, ActionMenuConfig, ActionMenuLayoutConfig, ActionMenuSectionConfig,
};

pub(super) struct ActionMenuDropdown {
    edge: BarEdge,
    region: BarRegion,
    bar_sender: relm4::Sender<BarMsg>,
    config: ActionMenuConfig,
}

pub(super) struct ActionMenuDropdownInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
    pub(super) bar_sender: relm4::Sender<BarMsg>,
    pub(super) config: ActionMenuConfig,
}

#[derive(Debug)]
pub(super) enum ActionMenuDropdownInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    Run(WidgetAction),
}

fn configure_panel(scroller: &gtk::ScrolledWindow, content: &gtk::Box, config: &ActionMenuConfig) {
    content.set_spacing(config.layout.row_spacing.max(0));

    if let Some(width) = config.panel.width {
        let width = width.max(1);
        scroller.set_min_content_width(width);
        content.set_width_request(width);
    }

    if let Some(max_height) = config.panel.max_height {
        scroller.set_max_content_height(max_height.max(1));
    }
}

fn build_header(
    config: &ActionMenuConfig,
    sender: &ComponentSender<ActionMenuDropdown>,
) -> gtk::Box {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.add_css_class("dropdown-header");
    header.add_css_class("action-menu-header");
    header.set_hexpand(true);

    let title = gtk::Label::new(Some("Dashboard"));
    title.add_css_class("dropdown-title");
    title.add_css_class("action-menu-section-title");
    title.set_halign(gtk::Align::Start);
    title.set_hexpand(true);
    header.append(&title);

    let settings = ActionMenuActionConfig {
        label: "Settings".to_string(),
        icon: Some("\u{f013}".to_string()),
        action: ActionMenuActionKind::OpenSettings,
        command: None,
        args: Vec::new(),
        class: Some("action-menu-settings".to_string()),
        tooltip: Some("Settings".to_string()),
        show_label: false,
    };

    let power = ActionMenuActionConfig {
        label: "Power menu".to_string(),
        icon: Some("\u{f011}".to_string()),
        action: ActionMenuActionKind::Command,
        command: Some(config.header.power_command.clone()),
        args: config.header.power_args.clone(),
        class: Some("action-menu-power".to_string()),
        tooltip: Some("Power menu".to_string()),
        show_label: false,
    };

    header.append(&build_action_button(&settings, &config.layout, sender));
    header.append(&build_action_button(&power, &config.layout, sender));

    header
}

fn render_sections(
    content: &gtk::Box,
    config: &ActionMenuConfig,
    sender: &ComponentSender<ActionMenuDropdown>,
) {
    for section in &config.sections {
        content.append(&build_section(section, &config.layout, sender));
    }
}

fn build_section(
    section: &ActionMenuSectionConfig,
    layout: &ActionMenuLayoutConfig,
    sender: &ComponentSender<ActionMenuDropdown>,
) -> gtk::Box {
    let section_box = gtk::Box::new(gtk::Orientation::Vertical, layout.row_spacing.max(0));
    section_box.add_css_class("action-menu-section");
    section_box.add_css_class("action-menu-card-section");

    let grid = gtk::Grid::new();
    grid.add_css_class("action-menu-actions");
    grid.set_halign(gtk::Align::Fill);
    grid.set_hexpand(true);
    grid.set_column_homogeneous(true);
    grid.set_row_spacing(layout.row_spacing.max(0) as u32);

    let columns = section.columns.unwrap_or(layout.columns).max(1);

    for (index, action) in section.actions.iter().enumerate() {
        let control = build_action_control(action, layout, sender);
        control.set_hexpand(true);
        control.set_halign(gtk::Align::Center);
        grid.attach(
            &control,
            (index % columns) as i32,
            (index / columns) as i32,
            1,
            1,
        );
    }

    if let Some(title) = &section.title {
        let title_label = gtk::Label::new(Some(title));
        title_label.add_css_class("action-menu-section-title");
        title_label.set_halign(gtk::Align::Start);
        section_box.append(&title_label);
    }

    section_box.append(&grid);

    section_box
}

fn widget_action(action: &ActionMenuActionConfig) -> Option<WidgetAction> {
    match action.action {
        ActionMenuActionKind::Command => {
            let Some(program) = action.command.clone() else {
                tracing::error!("Ignoring action menu command without a program");
                return None;
            };

            Some(WidgetAction::ActionMenu(ActionMenuAction::Run {
                command: ActionMenuCommand {
                    program,
                    args: action.args.clone(),
                },
            }))
        }
        ActionMenuActionKind::OpenSettings => {
            Some(WidgetAction::ActionMenu(ActionMenuAction::OpenSettings))
        }
    }
}

fn build_action_control(
    action: &ActionMenuActionConfig,
    layout: &ActionMenuLayoutConfig,
    sender: &ComponentSender<ActionMenuDropdown>,
) -> gtk::Widget {
    let button = build_action_button(action, layout, sender);

    if action.show_label && !action.label.is_empty() && action.icon.is_some() {
        let item = gtk::Box::new(gtk::Orientation::Vertical, 6);
        item.add_css_class("action-menu-action-item");
        item.set_halign(gtk::Align::Center);
        item.set_valign(gtk::Align::Center);
        item.set_width_request(layout.button_width.unwrap_or(44).max(1));

        let label = gtk::Label::new(Some(&action.label));
        label.add_css_class("action-menu-action-label");
        label.set_halign(gtk::Align::Center);

        item.append(&button);
        item.append(&label);

        item.upcast()
    } else {
        button.upcast()
    }
}

fn build_action_button(
    action: &ActionMenuActionConfig,
    layout: &ActionMenuLayoutConfig,
    sender: &ComponentSender<ActionMenuDropdown>,
) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("flat");

    match &action.class {
        Some(class) => button.add_css_class(class),
        None => button.add_css_class("action-menu-action"),
    }

    button.set_cursor_from_name(Some("pointer"));
    button.set_tooltip_text(
        action
            .tooltip
            .as_deref()
            .or_else(|| (!action.label.is_empty()).then_some(action.label.as_str())),
    );

    if let Some(width) = layout.button_width {
        button.set_width_request(width.max(1));
    }

    if let Some(height) = layout.button_height {
        button.set_height_request(height.max(1));
    }

    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    content.add_css_class("action-menu-button-content");
    content.set_halign(gtk::Align::Center);
    content.set_valign(gtk::Align::Center);

    if let Some(icon) = &action.icon {
        let icon_label = gtk::Label::new(Some(icon));
        icon_label.add_css_class("action-menu-action-icon");
        icon_label.set_halign(gtk::Align::Center);
        icon_label.set_valign(gtk::Align::Center);
        content.append(&icon_label);
    }

    if action.show_label && !action.label.is_empty() && action.icon.is_none() {
        let label = gtk::Label::new(Some(&action.label));
        label.add_css_class("action-menu-action-label");
        label.set_halign(gtk::Align::Center);
        content.append(&label);
    }

    button.set_child(Some(&content));

    if let Some(action) = widget_action(action) {
        let input_sender = sender.input_sender().clone();

        button.connect_clicked(move |_| {
            let _ = input_sender.send(ActionMenuDropdownInput::Run(action.clone()));
        });
    } else {
        button.set_sensitive(false);
    }

    button
}

#[relm4::component(pub(super))]
impl Component for ActionMenuDropdown {
    type Init = ActionMenuDropdownInit;
    type Input = ActionMenuDropdownInput;
    type Output = ();
    type CommandOutput = ();

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

                #[name = "scroller"]
                gtk::ScrolledWindow {
                    add_css_class: "action-menu-scroll",
                    set_policy: (gtk::PolicyType::Never, gtk::PolicyType::Automatic),
                    set_propagate_natural_height: true,
                }
            }
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
            config: init.config,
        };

        let widgets = view_output!();

        let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
        content.add_css_class("dropdown-content");
        content.add_css_class("action-menu-dropdown-content");

        configure_panel(&widgets.scroller, &content, &model.config);
        content.append(&build_header(&model.config, &sender));
        render_sections(&content, &model.config, &sender);
        widgets.scroller.set_child(Some(&content));

        dropdown::connect_revealer(&widgets.popover, &widgets.revealer);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            ActionMenuDropdownInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
            }
            ActionMenuDropdownInput::Run(action) => {
                widgets.popover.popdown();

                let _ = self.bar_sender.send(BarMsg::WidgetEvent(WidgetEvent {
                    widget_id: "action_menu",
                    action,
                }));
            }
        }

        self.update_view(widgets, sender);
    }
}
