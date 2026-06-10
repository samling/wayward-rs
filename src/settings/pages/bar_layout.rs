use crate::config::{BarConfig, BarRegionKey};
use relm4::{
    gtk::{self, prelude::*},
    prelude::ComponentSender,
};

use super::super::window::{SettingsInput, SettingsWindow};

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
    sender: &ComponentSender<SettingsWindow>,
) {
    for (index, bar) in bars.iter().enumerate() {
        render_bar_layout_section(container, index, bar, sender);
    }
}

fn render_bar_layout_section(
    container: &gtk::Box,
    index: usize,
    bar: &BarConfig,
    sender: &ComponentSender<SettingsWindow>,
) {
    let section_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    section_box.add_css_class("settings-section");

    let title_text = bar
        .name
        .clone()
        .unwrap_or_else(|| format!("Bar {}", index + 1));

    let title = gtk::Label::new(Some(&title_text));
    title.set_halign(gtk::Align::Start);
    title.add_css_class("settings-section-title");
    section_box.append(&title);

    let group = gtk::Box::new(gtk::Orientation::Vertical, 12);
    group.add_css_class("settings-group");

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

    token
}

fn available_widget_ids() -> Vec<&'static str> {
    crate::bar::registry::WIDGETS
        .iter()
        .map(|widget| widget.id())
        .collect()
}
