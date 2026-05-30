use crate::config::BarConfig;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BarItem {
    Workspaces,
    Clock,
    Battery,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BarLayout {
    pub(super) left: Vec<BarItem>,
    pub(super) center: Vec<BarItem>,
    pub(super) right: Vec<BarItem>,
}

impl BarLayout {
    pub(super) fn default_top_bar() -> Self {
        Self {
            left: vec![BarItem::Workspaces],
            center: vec![],
            right: vec![BarItem::Clock, BarItem::Battery],
        }
    }

    pub(super) fn unique_items(&self) -> Vec<BarItem> {
        let mut unique = Vec::new();

        for item in self.items() {
            if !unique.contains(&item) {
                unique.push(item);
            }
        }

        unique
    }

    pub(super) fn items(&self) -> Vec<BarItem> {
        let mut items = Vec::new();

        items.extend(self.left.iter().copied());
        items.extend(self.center.iter().copied());
        items.extend(self.right.iter().copied());

        items
    }

    pub(super) fn from_config(config: Option<&BarConfig>) -> Self {
        let default = Self::default_top_bar();

        let Some(config) = config else {
            return default;
        };

        Self {
            left: config
                .left
                .as_ref()
                .map_or(default.left, |items| parse_items(items)),
            center: config
                .center
                .as_ref()
                .map_or(default.center, |items| parse_items(items)),
            right: config
                .right
                .as_ref()
                .map_or(default.right, |items| parse_items(items)),
        }
    }
}

fn parse_items(items: &[String]) -> Vec<BarItem> {
    items.iter().filter_map(|item| parse_item(item)).collect()
}

fn parse_item(item: &str) -> Option<BarItem> {
    match item {
        "workspaces" => Some(BarItem::Workspaces),
        "clock" => Some(BarItem::Clock),
        "battery" => Some(BarItem::Battery),
        unknown => {
            tracing::error!("Unknown bar item in config: {unknown}");
            None
        }
    }
}

#[test]
fn unique_items_keeps_first_occurrence_order() {
    let layout = BarLayout {
        left: vec![BarItem::Clock],
        center: vec![BarItem::Workspaces, BarItem::Clock],
        right: vec![BarItem::Battery, BarItem::Workspaces],
    };

    assert_eq!(
        layout.unique_items(),
        vec![BarItem::Clock, BarItem::Workspaces, BarItem::Battery]
    );
}
