use super::registry;
use super::widget::WidgetInstance;
use crate::config::BarConfig;
use relm4::gtk;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BarEdge {
    Top,
    Bottom,
    Left,
    Right,
}

impl BarEdge {
    pub(crate) fn from_config(value: Option<&str>) -> Self {
        match value {
            Some("top") | None => Self::Top,
            Some("bottom") => Self::Bottom,
            Some("left") => Self::Left,
            Some("right") => Self::Right,
            Some(unknown) => {
                tracing::error!("Unknown bar edge in config: {unknown}");
                Self::Top
            }
        }
    }

    pub(crate) fn is_vertical(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    pub(crate) fn orientation(self) -> gtk::Orientation {
        if self.is_vertical() {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        }
    }

    pub(crate) fn center_halign(self) -> gtk::Align {
        if self.is_vertical() {
            gtk::Align::Fill
        } else {
            gtk::Align::Center
        }
    }

    pub(crate) fn center_valign(self) -> gtk::Align {
        if self.is_vertical() {
            gtk::Align::Center
        } else {
            gtk::Align::Fill
        }
    }

    pub(crate) fn center_hexpand(self) -> bool {
        !self.is_vertical()
    }

    pub(crate) fn center_vexpand(self) -> bool {
        self.is_vertical()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BarLayout {
    pub(super) start: Vec<WidgetInstance>,
    pub(super) center: Vec<WidgetInstance>,
    pub(super) end: Vec<WidgetInstance>,
}

impl BarLayout {
    pub(super) fn default_top_bar(app_config: &crate::config::AppConfig) -> Self {
        Self {
            start: vec![parse_item(app_config, "workspaces").unwrap()],
            center: vec![],
            end: vec![
                parse_item(app_config, "clock").unwrap(),
                parse_item(app_config, "battery").unwrap(),
            ],
        }
    }

    pub(super) fn from_config(
        app_config: &crate::config::AppConfig,
        config: Option<&BarConfig>,
    ) -> Self {
        let default = Self::default_top_bar(app_config);

        let Some(config) = config else {
            return default;
        };

        Self {
            start: config
                .start
                .as_ref()
                .map_or(default.start, |items| parse_items(app_config, items)),
            center: config
                .center
                .as_ref()
                .map_or(default.center, |items| parse_items(app_config, items)),
            end: config
                .end
                .as_ref()
                .map_or(default.end, |items| parse_items(app_config, items)),
        }
    }
}

fn parse_items(app_config: &crate::config::AppConfig, items: &[String]) -> Vec<WidgetInstance> {
    items
        .iter()
        .filter_map(|item| parse_item(app_config, item))
        .collect()
}

fn parse_item(app_config: &crate::config::AppConfig, reference: &str) -> Option<WidgetInstance> {
    let (widget_type, instance) = split_widget_ref(reference);

    let Some(widget) = registry::widget_by_id(widget_type) else {
        tracing::error!("Unknown bar widget in config: {widget_type}");
        return None;
    };

    Some(WidgetInstance {
        id: reference.to_string(),
        widget_type: widget_type.to_string(),
        widget,
        config: resolved_config(app_config, widget_type, instance),
    })
}

fn split_widget_ref(reference: &str) -> (&str, Option<&str>) {
    match reference.split_once('.') {
        Some((widget_type, instance)) => (widget_type, Some(instance)),
        None => (reference, None),
    }
}

fn resolved_config(
    app_config: &crate::config::AppConfig,
    widget_type: &str,
    instance: Option<&str>,
) -> toml::value::Table {
    let mut resolved = toml::value::Table::new();

    let Some(type_table) = app_config.widgets.get(widget_type) else {
        return resolved;
    };

    for (key, value) in type_table {
        if !value.is_table() {
            resolved.insert(key.clone(), value.clone());
        }
    }

    if let Some(instance) = instance {
        if let Some(instance_table) = type_table.get(instance).and_then(|value| value.as_table()) {
            for (key, value) in instance_table {
                resolved.insert(key.clone(), value.clone());
            }
        }
    }

    resolved
}
