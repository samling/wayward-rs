use relm4::{Component, ComponentController, ComponentSender, Controller, gtk, gtk::prelude::*};

use super::super::{
    page::{SettingsConfig, render_section},
    window::SettingsWindow,
};
use super::action_menu_editor::{ActionMenuEditor, ActionMenuEditorInit};

pub(crate) fn render(
    container: &gtk::Box,
    config: &SettingsConfig,
    sender: &ComponentSender<SettingsWindow>,
    editor: &mut Option<Controller<ActionMenuEditor>>,
) {
    let controller = ActionMenuEditor::builder()
        .launch(ActionMenuEditorInit {
            sections: read_sections(config),
            input_sender: sender.input_sender().clone(),
        })
        .detach();
    container.append(controller.widget());
    *editor = Some(controller);

    for section in super::widgets::config_sections("action_menu", &config.widgets) {
        render_section(container, section, sender);
    }

    render_section(
        container,
        super::style_sections::section("Action menu", &config.style),
        sender,
    );
}

fn read_sections(config: &SettingsConfig) -> Vec<toml::value::Table> {
    match config
        .widgets
        .get("action_menu")
        .and_then(|table| table.get("sections"))
    {
        // Configured (including an explicit empty list): show exactly what's set.
        Some(value) => value
            .as_array()
            .map(|sections| {
                sections
                    .iter()
                    .filter_map(|section| section.as_table().cloned())
                    .collect()
            })
            .unwrap_or_default(),
        // Never configured: show the built-in defaults.
        None => crate::bar::widgets::action_menu::default_sections()
            .iter()
            .filter_map(|section| section.as_table().cloned())
            .collect(),
    }
}
