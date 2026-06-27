use relm4::{
    ComponentParts, ComponentSender, SimpleComponent,
    factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque},
    gtk,
    gtk::prelude::*,
};
use std::{cell::RefCell, rc::Rc, time::Duration};

use super::super::window::SettingsInput;
use crate::config::ActionMenuSectionMove;

const ACTION_MENU_EDITOR_WRITE_DEBOUNCE: Duration = Duration::from_millis(500);

pub(crate) struct ActionMenuEditor {
    _sections: FactoryVecDeque<ActionMenuSectionRow>,
    input_sender: relm4::Sender<SettingsInput>,
}

pub(crate) struct ActionMenuEditorInit {
    pub(crate) sections: Vec<toml::value::Table>,
    pub(crate) input_sender: relm4::Sender<SettingsInput>,
}

#[derive(Debug)]
pub(crate) enum ActionMenuEditorInput {
    SetSectionField {
        section: DynamicIndex,
        field: &'static str,
        value: Option<crate::config::ConfigValue>,
    },
    AddAction {
        section: DynamicIndex,
    },
    RemoveSection {
        section: DynamicIndex,
    },
    SetActionField {
        section: DynamicIndex,
        action: DynamicIndex,
        field: &'static str,
        value: Option<crate::config::ConfigValue>,
    },
    RemoveAction {
        section: DynamicIndex,
        action: DynamicIndex,
    },
    MoveSection {
        section: DynamicIndex,
        direction: ActionMenuSectionMove,
    },
}

pub(crate) struct ActionMenuEditorWidgets;

impl SimpleComponent for ActionMenuEditor {
    type Init = ActionMenuEditorInit;
    type Input = ActionMenuEditorInput;
    type Output = ();
    type Root = gtk::Box;
    type Widgets = ActionMenuEditorWidgets;

    fn init_root() -> Self::Root {
        gtk::Box::new(gtk::Orientation::Vertical, 10)
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.add_css_class("settings-section");

        let title = gtk::Label::new(Some("Buttons"));
        title.set_halign(gtk::Align::Start);
        title.add_css_class("settings-section-title");
        root.append(&title);

        if init.sections.is_empty() {
            let empty = gtk::Label::new(Some(
                "No sections configured. Add one in your config to edit buttons here.",
            ));
            empty.set_halign(gtk::Align::Start);
            empty.add_css_class("settings-row-description");
            root.append(&empty);
        }

        let section_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        root.append(&section_box);

        let mut sections = FactoryVecDeque::builder().launch(section_box).forward(
            sender.input_sender(),
            |output| match output {
                ActionMenuSectionOutput::SetField {
                    section,
                    field,
                    value,
                } => ActionMenuEditorInput::SetSectionField {
                    section,
                    field,
                    value,
                },
                ActionMenuSectionOutput::AddAction { section } => {
                    ActionMenuEditorInput::AddAction { section }
                }
                ActionMenuSectionOutput::Remove { section } => {
                    ActionMenuEditorInput::RemoveSection { section }
                }
                ActionMenuSectionOutput::SetActionField {
                    section,
                    action,
                    field,
                    value,
                } => ActionMenuEditorInput::SetActionField {
                    section,
                    action,
                    field,
                    value,
                },
                ActionMenuSectionOutput::RemoveAction { section, action } => {
                    ActionMenuEditorInput::RemoveAction { section, action }
                }
                ActionMenuSectionOutput::Move { section, direction } => {
                    ActionMenuEditorInput::MoveSection { section, direction }
                }
            },
        );

        {
            let mut sections = sections.guard();
            for section in init.sections {
                sections.push_back(ActionMenuSectionInit { section });
            }
        }

        let add_section = gtk::Button::with_label("Add section");
        add_section.set_halign(gtk::Align::Start);
        let input = init.input_sender.clone();
        add_section.connect_clicked(move |_| {
            let _ = input.send(SettingsInput::AddActionMenuSection);
        });
        root.append(&add_section);

        let model = Self {
            _sections: sections,
            input_sender: init.input_sender,
        };

        ComponentParts {
            model,
            widgets: ActionMenuEditorWidgets,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ActionMenuEditorInput::SetSectionField {
                section,
                field,
                value,
            } => {
                let _ = self
                    .input_sender
                    .send(SettingsInput::SetActionMenuSectionField {
                        section: section.current_index(),
                        field,
                        value,
                    });
            }
            ActionMenuEditorInput::AddAction { section } => {
                let _ = self.input_sender.send(SettingsInput::AddActionMenuAction {
                    section: section.current_index(),
                });
            }
            ActionMenuEditorInput::RemoveSection { section } => {
                let _ = self
                    .input_sender
                    .send(SettingsInput::RemoveActionMenuSection {
                        section: section.current_index(),
                    });
            }
            ActionMenuEditorInput::SetActionField {
                section,
                action,
                field,
                value,
            } => {
                let _ = self
                    .input_sender
                    .send(SettingsInput::SetActionMenuActionField {
                        section: section.current_index(),
                        action: action.current_index(),
                        field,
                        value,
                    });
            }
            ActionMenuEditorInput::RemoveAction { section, action } => {
                let _ = self
                    .input_sender
                    .send(SettingsInput::RemoveActionMenuAction {
                        section: section.current_index(),
                        action: action.current_index(),
                    });
            }
            ActionMenuEditorInput::MoveSection { section, direction } => {
                let _ = self
                    .input_sender
                    .send(SettingsInput::MoveActionMenuSection {
                        section: section.current_index(),
                        direction,
                    });
            }
        }
    }
}

struct ActionMenuSectionInit {
    section: toml::value::Table,
}

struct ActionMenuSectionRow {
    section: toml::value::Table,
    index: DynamicIndex,
    _actions: Option<FactoryVecDeque<ActionMenuActionRow>>,
}

#[derive(Debug)]
enum ActionMenuSectionInput {
    SetActionField {
        action: DynamicIndex,
        field: &'static str,
        value: Option<crate::config::ConfigValue>,
    },
    RemoveAction {
        action: DynamicIndex,
    },
}

#[derive(Debug)]
enum ActionMenuSectionOutput {
    SetField {
        section: DynamicIndex,
        field: &'static str,
        value: Option<crate::config::ConfigValue>,
    },
    AddAction {
        section: DynamicIndex,
    },
    Remove {
        section: DynamicIndex,
    },
    Move {
        section: DynamicIndex,
        direction: ActionMenuSectionMove,
    },
    SetActionField {
        section: DynamicIndex,
        action: DynamicIndex,
        field: &'static str,
        value: Option<crate::config::ConfigValue>,
    },
    RemoveAction {
        section: DynamicIndex,
        action: DynamicIndex,
    },
}

#[relm4::factory]
impl FactoryComponent for ActionMenuSectionRow {
    type Init = ActionMenuSectionInit;
    type Input = ActionMenuSectionInput;
    type Output = ActionMenuSectionOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[name = "root"]
        gtk::Box {
            add_css_class: "action-menu-settings-section",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 6,
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            section: init.section,
            index: index.clone(),
            _actions: None,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        populate_section_header(&widgets.root, index, &self.section, sender.clone());

        let action_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        widgets.root.append(&action_box);

        let mut actions = FactoryVecDeque::builder().launch(action_box).forward(
            sender.input_sender(),
            |output| match output {
                ActionMenuActionOutput::SetField {
                    action,
                    field,
                    value,
                } => ActionMenuSectionInput::SetActionField {
                    action,
                    field,
                    value,
                },
                ActionMenuActionOutput::Remove { action } => {
                    ActionMenuSectionInput::RemoveAction { action }
                }
            },
        );

        if let Some(action_values) = self
            .section
            .get("actions")
            .and_then(|value| value.as_array())
        {
            let mut actions = actions.guard();
            for action in action_values {
                if let Some(action) = action.as_table() {
                    actions.push_back(ActionMenuActionInit {
                        action: action.clone(),
                    });
                }
            }
        }

        let add_action = gtk::Button::with_label("Add button");
        add_action.set_halign(gtk::Align::Start);
        let output = sender.clone();
        let section = index.clone();
        add_action.connect_clicked(move |_| {
            let _ = output.output(ActionMenuSectionOutput::AddAction {
                section: section.clone(),
            });
        });
        widgets.root.append(&add_action);

        self._actions = Some(actions);

        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            ActionMenuSectionInput::SetActionField {
                action,
                field,
                value,
            } => {
                let _ = sender.output(ActionMenuSectionOutput::SetActionField {
                    section: self.index.clone(),
                    action,
                    field,
                    value,
                });
            }
            ActionMenuSectionInput::RemoveAction { action } => {
                let _ = sender.output(ActionMenuSectionOutput::RemoveAction {
                    section: self.index.clone(),
                    action,
                });
            }
        }
    }
}

fn populate_section_header(
    card: &gtk::Box,
    section_index: &DynamicIndex,
    section: &toml::value::Table,
    sender: FactorySender<ActionMenuSectionRow>,
) {
    let header_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let title_entry = gtk::Entry::new();
    title_entry.set_hexpand(true);
    title_entry.set_placeholder_text(Some("Section title"));
    title_entry.set_text(
        section
            .get("title")
            .and_then(|value| value.as_str())
            .unwrap_or(""),
    );
    {
        let output = sender.clone();
        let section = section_index.clone();
        connect_text_entry_commit(&title_entry, &title_entry.text(), move |text| {
            let value = (!text.is_empty()).then_some(crate::config::ConfigValue::String(text));
            let _ = output.output(ActionMenuSectionOutput::SetField {
                section: section.clone(),
                field: "title",
                value,
            });
        });
    }
    header_row.append(&labeled_control("Section title", &title_entry));

    let columns_spin = gtk::SpinButton::with_range(1.0, 8.0, 1.0);
    columns_spin.set_value(
        section
            .get("columns")
            .and_then(|value| value.as_integer())
            .unwrap_or(3) as f64,
    );
    {
        let output = sender.clone();
        let section = section_index.clone();
        columns_spin.connect_value_changed(move |spin| {
            let _ = output.output(ActionMenuSectionOutput::SetField {
                section: section.clone(),
                field: "columns",
                value: Some(crate::config::ConfigValue::Integer(spin.value() as i64)),
            });
        });
    }
    header_row.append(&labeled_control("Columns", &columns_spin));

    let move_up = gtk::Button::with_label("↑");
    move_up.set_focus_on_click(false);
    move_up.set_tooltip_text(Some("Move section up"));
    {
        let output = sender.clone();
        let section = section_index.clone();
        move_up.connect_clicked(move |_| {
            let _ = output.output(ActionMenuSectionOutput::Move {
                section: section.clone(),
                direction: ActionMenuSectionMove::Up,
            });
        });
    }
    header_row.append(&move_up);

    let move_down = gtk::Button::with_label("↓");
    move_down.set_focus_on_click(false);
    move_down.set_tooltip_text(Some("Move section down"));
    {
        let output = sender.clone();
        let section = section_index.clone();
        move_down.connect_clicked(move |_| {
            let _ = output.output(ActionMenuSectionOutput::Move {
                section: section.clone(),
                direction: ActionMenuSectionMove::Down,
            });
        });
    }
    header_row.append(&move_down);

    let remove_section = gtk::Button::with_label("Remove section");
    let output = sender;
    let section = section_index.clone();
    remove_section.connect_clicked(move |_| {
        let _ = output.output(ActionMenuSectionOutput::Remove {
            section: section.clone(),
        });
    });
    header_row.append(&remove_section);

    card.append(&header_row);
}

fn connect_text_entry_commit(
    entry: &gtk::Entry,
    current: &str,
    commit_text: impl Fn(String) + 'static,
) {
    let pending = Rc::new(RefCell::new(None::<gtk::glib::SourceId>));
    let last_committed = Rc::new(RefCell::new(current.to_string()));
    let commit_text: Rc<dyn Fn(String)> = Rc::new(commit_text);

    let commit_now: Rc<dyn Fn(&gtk::Entry)> = {
        let pending = pending.clone();
        let last_committed = last_committed.clone();
        let commit_text = commit_text.clone();
        Rc::new(move |entry| {
            cancel_pending_entry_commit(&pending);

            let text = entry.text().to_string();
            if text == *last_committed.borrow() {
                return;
            }

            *last_committed.borrow_mut() = text.clone();
            commit_text(text);
        })
    };

    {
        let pending = pending.clone();
        let commit_now = commit_now.clone();
        entry.connect_changed(move |entry| {
            cancel_pending_entry_commit(&pending);

            let entry = entry.clone();
            let pending_for_timeout = pending.clone();
            let commit_for_timeout = commit_now.clone();
            let source =
                gtk::glib::timeout_add_local_once(ACTION_MENU_EDITOR_WRITE_DEBOUNCE, move || {
                    *pending_for_timeout.borrow_mut() = None;
                    commit_for_timeout(&entry);
                });

            *pending.borrow_mut() = Some(source);
        });
    }

    {
        let commit_now = commit_now.clone();
        entry.connect_activate(move |entry| commit_now(entry));
    }

    let focus = gtk::EventControllerFocus::new();
    {
        let entry = entry.clone();
        let commit_now = commit_now.clone();
        focus.connect_leave(move |_| commit_now(&entry));
    }
    entry.add_controller(focus);
}

fn cancel_pending_entry_commit(pending: &Rc<RefCell<Option<gtk::glib::SourceId>>>) {
    if let Some(source) = pending.borrow_mut().take() {
        source.remove();
    }
}

fn labeled_control(label_text: &str, control: &impl IsA<gtk::Widget>) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    container.set_hexpand(control.as_ref().hexpands());
    container.set_valign(gtk::Align::Center);

    let label = gtk::Label::new(Some(label_text));
    label.set_halign(gtk::Align::Start);
    label.set_valign(gtk::Align::Center);
    label.set_width_chars(8);
    label.set_xalign(0.0);
    label.add_css_class("settings-row-label");
    container.append(&label);
    container.append(control);

    container
}

struct ActionMenuActionInit {
    action: toml::value::Table,
}

struct ActionMenuActionRow {
    action: toml::value::Table,
}

#[derive(Debug)]
enum ActionMenuActionOutput {
    SetField {
        action: DynamicIndex,
        field: &'static str,
        value: Option<crate::config::ConfigValue>,
    },
    Remove {
        action: DynamicIndex,
    },
}

#[relm4::factory]
impl FactoryComponent for ActionMenuActionRow {
    type Init = ActionMenuActionInit;
    type Input = ();
    type Output = ActionMenuActionOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[name = "root"]
        gtk::Box {
            add_css_class: "settings-row",
            add_css_class: "action-menu-settings-action",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            action: init.action,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        populate_action_row(&widgets.root, index, &self.action, sender);
        widgets
    }

    fn update(&mut self, _msg: Self::Input, _sender: FactorySender<Self>) {}
}

fn populate_action_row(
    outer: &gtk::Box,
    action_index: &DynamicIndex,
    action: &toml::value::Table,
    sender: FactorySender<ActionMenuActionRow>,
) {
    let str_field = |key: &str| {
        action
            .get(key)
            .and_then(|value| value.as_str())
            .unwrap_or("")
    };

    let top = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let icon = action_text_field(action_index, "icon", str_field("icon"), sender.clone());
    icon.set_width_chars(3);
    icon.set_hexpand(false);
    top.append(&labeled_control("Icon", &icon));

    let label = action_text_field(action_index, "label", str_field("label"), sender.clone());
    top.append(&labeled_control("Label", &label));

    let kind = action_kind_field(action_index, str_field("action"), sender.clone());
    top.append(&labeled_control("Action", &kind));
    outer.append(&top);

    let command = action_text_field(
        action_index,
        "command",
        str_field("command"),
        sender.clone(),
    );
    outer.append(&labeled_control("Command", &command));

    let bottom = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let args_text = action
        .get("args")
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    let args = action_args_field(action_index, &args_text, sender.clone());
    bottom.append(&labeled_control("Args", &args));

    let show_label = action
        .get("show-label")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    bottom.append(&action_toggle_field(
        action_index,
        show_label,
        sender.clone(),
    ));

    let remove = gtk::Button::with_label("Remove");
    let output = sender;
    let action = action_index.clone();
    remove.connect_clicked(move |_| {
        let _ = output.output(ActionMenuActionOutput::Remove {
            action: action.clone(),
        });
    });
    bottom.append(&remove);

    outer.append(&bottom);
}

fn action_text_field(
    action_index: &DynamicIndex,
    field: &'static str,
    current: &str,
    sender: FactorySender<ActionMenuActionRow>,
) -> gtk::Entry {
    let entry = gtk::Entry::new();
    entry.set_text(current);
    entry.set_hexpand(true);

    let output = sender;
    let action = action_index.clone();
    connect_text_entry_commit(&entry, current, move |text| {
        let value = (!text.is_empty()).then_some(crate::config::ConfigValue::String(text));
        let _ = output.output(ActionMenuActionOutput::SetField {
            action: action.clone(),
            field,
            value,
        });
    });

    entry
}

fn action_kind_field(
    action_index: &DynamicIndex,
    current: &str,
    sender: FactorySender<ActionMenuActionRow>,
) -> gtk::DropDown {
    let options = ["command", "open-settings"];
    let string_list = gtk::StringList::new(&["Command", "Open settings"]);
    let dropdown = gtk::DropDown::new(Some(string_list), None::<gtk::Expression>);
    let selected = options
        .iter()
        .position(|option| *option == current)
        .unwrap_or(0) as u32;
    dropdown.set_selected(selected);

    let output = sender;
    let action = action_index.clone();
    dropdown.connect_selected_notify(move |dropdown| {
        let value = options
            .get(dropdown.selected() as usize)
            .copied()
            .unwrap_or("command");
        let _ = output.output(ActionMenuActionOutput::SetField {
            action: action.clone(),
            field: "action",
            value: Some(crate::config::ConfigValue::String(value.to_string())),
        });
    });

    dropdown
}

fn action_toggle_field(
    action_index: &DynamicIndex,
    current: bool,
    sender: FactorySender<ActionMenuActionRow>,
) -> gtk::CheckButton {
    let check = gtk::CheckButton::with_label("Show label");
    check.set_active(current);

    let output = sender;
    let action = action_index.clone();
    check.connect_toggled(move |check| {
        let _ = output.output(ActionMenuActionOutput::SetField {
            action: action.clone(),
            field: "show-label",
            value: Some(crate::config::ConfigValue::Bool(check.is_active())),
        });
    });

    check
}

fn action_args_field(
    action_index: &DynamicIndex,
    current: &str,
    sender: FactorySender<ActionMenuActionRow>,
) -> gtk::Entry {
    let entry = gtk::Entry::new();
    entry.set_text(current);
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("args (space separated)"));

    let output = sender;
    let action = action_index.clone();
    connect_text_entry_commit(&entry, current, move |text| {
        let args: Vec<String> = text.split_whitespace().map(ToOwned::to_owned).collect();
        let value = (!args.is_empty()).then_some(crate::config::ConfigValue::StringList(args));
        let _ = output.output(ActionMenuActionOutput::SetField {
            action: action.clone(),
            field: "args",
            value,
        });
    });

    entry
}
