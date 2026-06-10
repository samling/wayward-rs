use crate::config::{BarConfig, BarRegionKey};
use relm4::{
    gtk::{self, glib::prelude::{StaticType, ToValue}, prelude::*},
    prelude::ComponentSender,
};

use super::super::window::{SettingsInput, SettingsWindow};

const BAR_EDGES: &[&str] = &["top", "bottom", "left", "right"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BarRegionView {
    Start,
    Center,
    End,
}

impl BarRegionView {
    fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Center => "Center",
            Self::End => "End",
        }
    }

    fn key(self) -> BarRegionKey {
        match self {
            Self::Start => BarRegionKey::Start,
            Self::Center => BarRegionKey::Center,
            Self::End => BarRegionKey::End,
        }
    }

    fn widgets<'a>(self, bar: &'a BarConfig) -> Option<&'a [String]> {
        match self {
            Self::Start => bar.start.as_deref(),
            Self::Center => bar.center.as_deref(),
            Self::End => bar.end.as_deref(),
        }
    }
}

pub(crate) fn render(
    container: &gtk::Box,
    bars: &[BarConfig],
    available_monitors: &[String],
    sender: &ComponentSender<SettingsWindow>,
) {
    container.append(&add_bar_row(sender));

    let can_remove = bars.len() > 1;

    for (index, bar) in bars.iter().enumerate() {
        render_bar_layout_section(
            container,
            index,
            bar,
            available_monitors,
            can_remove,
            sender
        );
    }
}

fn add_bar_row(sender: &ComponentSender<SettingsWindow>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("settings-row");

    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("New bar name"));
    entry.set_hexpand(true);
    row.append(&entry);

    let add = gtk::Button::with_label("Add bar");
    add.add_css_class("suggested-action");
    row.append(&add);

    let sender_add = sender.input_sender().clone();
    add.connect_clicked(move |_| {
        let name = entry.text().trim().to_string();

        if name.is_empty() {
            return;
        }

        let _ = sender_add.send(SettingsInput::AddBar { name });
        entry.set_text("");
    });

    row
}

fn render_bar_layout_section(
    container: &gtk::Box,
    index: usize,
    bar: &BarConfig,
    available_monitors: &[String],
    can_remove: bool,
    sender: &ComponentSender<SettingsWindow>,
) {
    let section_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    section_box.add_css_class("settings-section");

    let title_text = bar
        .name
        .clone()
        .unwrap_or_else(|| format!("Bar {}", index + 1));

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let title = gtk::Label::new(Some(&title_text));
    title.set_halign(gtk::Align::Start);
    title.set_hexpand(true);
    title.add_css_class("settings-section-title");
    header.append(&title);

    if let Some(bar_name) = bar.name.clone() {
        let remove = gtk::Button::from_icon_name("user-trash-symbolic");
        remove.add_css_class("flat");
        remove.set_tooltip_markup(Some("Remove bar"));
        remove.set_sensitive(can_remove);

        let sender_remove = sender.input_sender().clone();
        remove.connect_clicked(move |_| {
            let _ = sender_remove.send(SettingsInput::RemoveBar {
                name: bar_name.clone(),
            });
        });

        header.append(&remove);
    }

    section_box.append(&header);

    let group = gtk::Box::new(gtk::Orientation::Vertical, 12);
    group.add_css_class("settings-group");

    group.append(&bar_name_row(bar.name.as_deref(), sender));
    group.append(&bar_edge_row(bar.name.as_deref(), bar.edge.as_deref(), sender));

    group.append(&bar_monitors_row(
        bar.name.as_deref(),
        bar.monitors.as_deref(),
        available_monitors,
        sender,
    ));

    for region in [
        BarRegionView::Start,
        BarRegionView::Center,
        BarRegionView::End,
    ] {
        let widgets = region.widgets(bar).unwrap_or(&[]);
        group.append(&bar_region_row(
            bar.name.as_deref(),
            region,
            widgets,
            sender,
        ));
    }

    section_box.append(&group);
    container.append(&section_box);
}

fn bar_name_row(
    bar_name: Option<&str>,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("settings-row");
    row.add_css_class("bar-name-settings-row");

    let label = gtk::Label::new(Some("Name"));
    label.set_halign(gtk::Align::Start);
    label.set_width_chars(8);
    label.add_css_class("settings-row-label");
    row.append(&label);

    let entry = gtk::Entry::new();
    entry.set_hexpand(true);
    entry.set_text(bar_name.unwrap_or(""));
    entry.set_sensitive(bar_name.is_some());
    row.append(&entry);

    let rename = gtk::Button::with_label("Rename");
    rename.set_sensitive(false);
    row.append(&rename);

    let current_name_for_change = bar_name.map(str::to_string);
    let rename_for_change = rename.clone();

    entry.connect_changed(move |entry| {
        let Some(current_name) = &current_name_for_change else {
            rename_for_change.set_sensitive(false);
            return;
        };

        let next_name = entry.text().trim().to_string();
        rename_for_change.set_sensitive(!next_name.is_empty() && next_name != *current_name);
    });

    let current_name = bar_name.map(str::to_string);
    let sender_rename = sender.input_sender().clone();

    rename.connect_clicked(move |_| {
        let Some(current_name) = &current_name else {
            return;
        };

        let next_name = entry.text().trim().to_string();

        if next_name.is_empty() || next_name == *current_name {
            return;
        }

        let _ = sender_rename.send(SettingsInput::RenameBar {
            current_name: current_name.clone(),
            next_name,
        });
    });

    row
}

fn bar_edge_row(
    bar_name: Option<&str>,
    edge: Option<&str>,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("settings-row");
    row.add_css_class("bar-edge-settings-row");

    let label = gtk::Label::new(Some("Edge"));
    label.set_halign(gtk::Align::Start);
    label.set_width_chars(8);
    label.add_css_class("settings-row-label");
    row.append(&label);

    let string_list = gtk::StringList::new(BAR_EDGES);
    let dropdown = gtk::DropDown::new(Some(string_list), None::<gtk::Expression>);
    dropdown.set_sensitive(bar_name.is_some());

    let selected = edge
        .and_then(|edge| BAR_EDGES.iter().position(|candidate| *candidate == edge))
        .unwrap_or(0);

    dropdown.set_selected(selected as u32);
    row.append(&dropdown);

    let bar_name_edge = bar_name.map(str::to_string);
    let sender_edge = sender.input_sender().clone();

    dropdown.connect_selected_notify(move |dropdown| {
        let Some(bar_name) = &bar_name_edge else {
            return;
        };

        let selected = dropdown.selected();

        if selected == gtk::INVALID_LIST_POSITION {
            return;
        }

        let Some(edge) = BAR_EDGES.get(selected as usize) else {
            return;
        };

        let _ = sender_edge.send(SettingsInput::SetBarEdge {
            bar_name: bar_name.clone(),
            edge: (*edge).to_string(),
        });
    });

    row
}

fn bar_monitors_row(
    bar_name: Option<&str>,
    monitors: Option<&[String]>,
    available_monitors: &[String],
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let monitors = monitors.unwrap_or(&[]);

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("settings-row");
    row.add_css_class("bar-monitor-settings-row");

    let label = gtk::Label::new(Some("Monitors"));
    label.set_halign(gtk::Align::Start);
    label.set_width_chars(8);
    label.add_css_class("settings-row-label");
    row.append(&label);

    let values = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    values.set_hexpand(true);
    values.add_css_class("bar-region-widget-list");

    if monitors.is_empty() {
        let all = gtk::Label::new(Some("All monitors"));
        all.add_css_class("settings-row-value");
        values.append(&all);
    } else {
        for index in 0..monitors.len() {
            values.append(&monitor_token(bar_name, monitors, index, sender));
        }
    }

    row.append(&values);

    let mut options = vec!["Add monitor".to_string()];
    options.extend(available_monitors.iter().cloned());

    let string_list = gtk::StringList::new(&options.iter().map(String::as_str).collect::<Vec<_>>());
    let add = gtk::DropDown::new(Some(string_list), None::<gtk::Expression>);
    add.add_css_class("bar-monitor-add");
    add.set_sensitive(bar_name.is_some() && !available_monitors.is_empty());
    add.set_selected(0);
    row.append(&add);

    let bar_name_add = bar_name.map(str::to_string);
    let monitors_add = monitors.to_vec();
    let available_monitors_add = available_monitors.to_vec();
    let sender_add = sender.input_sender().clone();

    add.connect_selected_notify(move |dropdown| {
        let Some(bar_name) = &bar_name_add else {
            return;
        };

        let selected = dropdown.selected();

        if selected == 0 || selected == gtk::INVALID_LIST_POSITION {
            return;
        }

        let Some(monitor) = available_monitors_add.get((selected - 1) as usize) else {
            return;
        };

        let mut monitors = monitors_add.clone();

        if !monitors.contains(monitor) {
            monitors.push(monitor.clone());
        }

        let _ = sender_add.send(SettingsInput::SetBarMonitors { bar_name: bar_name.clone(), monitors });

        dropdown.set_selected(0);
    });

    row
}

fn monitor_token(
    bar_name: Option<&str>,
    monitors: &[String],
    index: usize,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let token = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    token.add_css_class("bar-widget-token");

    let label = gtk::Label::new(Some(&monitors[index]));
    label.add_css_class("bar-widget-token-label");
    token.append(&label);

    let remove = gtk::Button::from_icon_name("window-close-symbolic");
    remove.add_css_class("flat");
    remove.add_css_class("bar-widget-token-button");
    remove.set_sensitive(bar_name.is_some());
    token.append(&remove);

    let bar_name_remove = bar_name.map(str::to_string);
    let monitors_remove = monitors.to_vec();
    let sender_remove = sender.input_sender().clone();

    remove.connect_clicked(move |_| {
        let Some(bar_name) = &bar_name_remove else {
            return;
        };

        let mut monitors = monitors_remove.clone();
        monitors.remove(index);

        let _ = sender_remove.send(SettingsInput::SetBarMonitors {
            bar_name: bar_name.clone(),
            monitors,
        });
    });

    token
}

fn bar_region_row(
    bar_name: Option<&str>,
    region: BarRegionView,
    widgets: &[String],
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("settings-row");
    row.add_css_class("bar-region-settings-row");

    let label = gtk::Label::new(Some(region.label()));
    label.set_halign(gtk::Align::Start);
    label.set_width_chars(8);
    label.add_css_class("settings-row-label");
    row.append(&label);

    let values = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    values.set_hexpand(true);
    values.add_css_class("bar-region-widget-list");

    for index in 0..widgets.len() {
        values.append(&bar_widget_token(bar_name, region, widgets, index, sender));
    }

    if widgets.is_empty() {
        let empty = gtk::Label::new(Some("Empty"));
        empty.add_css_class("settings-row-value");
        values.append(&empty);
    }

    row.append(&values);

    let mut options = vec!["Add widget".to_string()];
    options.extend(available_widget_ids().into_iter().map(str::to_string));

    let string_list = gtk::StringList::new(&options.iter().map(String::as_str).collect::<Vec<_>>());
    let add = gtk::DropDown::new(Some(string_list), None::<gtk::Expression>);
    add.add_css_class("bar-widget-add");
    add.set_sensitive(bar_name.is_some());
    add.set_selected(0);
    row.append(&add);

    let bar_name_add = bar_name.map(str::to_string);
    let widgets_add = widgets.to_vec();
    let sender_add = sender.input_sender().clone();

    add.connect_selected_notify(move |dropdown| {
        let Some(bar_name) = &bar_name_add else {
            return;
        };

        let selected = dropdown.selected();

        if selected == 0 || selected == gtk::INVALID_LIST_POSITION {
            return;
        }

        let widget_ids = available_widget_ids();
        let Some(widget_id) = widget_ids.get((selected - 1) as usize) else {
            return;
        };

        let mut widgets = widgets_add.clone();
        widgets.push((*widget_id).to_string());

        let _ = sender_add.send(SettingsInput::SetBarRegion {
            bar_name: bar_name.clone(),
            region: region.key(),
            widgets,
        });

        dropdown.set_selected(0);
    });

    row
}

fn bar_widget_token(
    bar_name: Option<&str>,
    region: BarRegionView,
    widgets: &[String],
    index: usize,
    sender: &ComponentSender<SettingsWindow>,
) -> gtk::Box {
    let token = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    token.add_css_class("bar-widget-token");

    let label = gtk::Label::new(Some(&widgets[index]));
    label.add_css_class("bar-widget-token-label");
    token.append(&label);

    let remove = gtk::Button::from_icon_name("window-close-symbolic");
    remove.add_css_class("flat");
    remove.add_css_class("bar-widget-token-button");
    remove.set_sensitive(bar_name.is_some());
    token.append(&remove);

    let bar_name_remove = bar_name.map(str::to_string);
    let widgets_remove = widgets.to_vec();
    let sender_remove = sender.input_sender().clone();

    remove.connect_clicked(move |_| {
        let Some(bar_name) = &bar_name_remove else {
            return;
        };

        let mut widgets = widgets_remove.clone();
        widgets.remove(index);

        let _ = sender_remove.send(SettingsInput::SetBarRegion {
            bar_name: bar_name.clone(),
            region: region.key(),
            widgets,
        });
    });

    if bar_name.is_some() {
        let drag = gtk::DragSource::new();
        drag.set_actions(gtk::gdk::DragAction::MOVE);

        drag.connect_prepare(move |_, _, _| {
            Some(gtk::gdk::ContentProvider::for_value(&(index as u32).to_value()))
        });

        token.add_controller(drag);

        let drop = gtk::DropTarget::new(u32::static_type(), gtk::gdk::DragAction::MOVE);

        let bar_name_drop = bar_name.map(str::to_string);
        let widgets_drop = widgets.to_vec();
        let sender_drop = sender.input_sender().clone();

        drop.connect_drop(move |_, value, _, _| {
            let Ok(from) = value.get::<u32>() else {
                return false;
            };

            let from = from as usize;

            let Some(bar_name) = &bar_name_drop else {
                return false;
            };

            let widgets = move_widget(&widgets_drop, from, index);

            let _ = sender_drop.send(SettingsInput::SetBarRegion {
                bar_name: bar_name.clone(),
                region: region.key(),
                widgets,
            });

            true
        });

        token.add_controller(drop)
    }

    token
}

fn available_widget_ids() -> Vec<&'static str> {
    crate::bar::registry::WIDGETS
        .iter()
        .map(|widget| widget.id())
        .collect()
}

fn move_widget(widgets: &[String], from: usize, to: usize) -> Vec<String> {
    let mut reordered = widgets.to_vec();

    if from >= reordered.len() || to >= reordered.len() || from == to {
        return reordered;
    }

    let widget = reordered.remove(from);
    reordered.insert(to, widget);
    reordered
}
