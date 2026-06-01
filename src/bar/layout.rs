use super::registry;
use super::widget::BarWidget;
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

#[derive(Clone)]
pub(crate) struct BarLayout {
    pub(super) start: Vec<&'static dyn BarWidget>,
    pub(super) center: Vec<&'static dyn BarWidget>,
    pub(super) end: Vec<&'static dyn BarWidget>,
}

impl BarLayout {
    pub(super) fn default_top_bar() -> Self {
        Self {
            start: vec![registry::widget_by_id("workspaces").unwrap()],
            center: vec![],
            end: vec![
                registry::widget_by_id("clock").unwrap(),
                registry::widget_by_id("battery").unwrap(),
            ],
        }
    }

    pub(super) fn from_config(config: Option<&BarConfig>) -> Self {
        let default = Self::default_top_bar();

        let Some(config) = config else {
            return default;
        };

        Self {
            start: config
                .start
                .as_ref()
                .map_or(default.start, |items| parse_items(items)),
            center: config
                .center
                .as_ref()
                .map_or(default.center, |items| parse_items(items)),
            end: config
                .end
                .as_ref()
                .map_or(default.end, |items| parse_items(items)),
        }
    }
}

impl std::fmt::Debug for BarLayout {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BarLayout")
            .field("start", &widget_ids(&self.start))
            .field("center", &widget_ids(&self.center))
            .field("end", &widget_ids(&self.end))
            .finish()
    }
}

fn widget_ids(widgets: &[&'static dyn BarWidget]) -> Vec<&'static str> {
    widgets.iter().map(|widget| widget.id()).collect()
}

fn parse_items(items: &[String]) -> Vec<&'static dyn BarWidget> {
    items.iter().filter_map(|item| parse_item(item)).collect()
}

fn parse_item(item: &str) -> Option<&'static dyn BarWidget> {
    let Some(bar_item) = registry::widget_by_id(item) else {
        tracing::error!("Unknown bar item in config: {item}");
        return None;
    };

    Some(bar_item)
}
