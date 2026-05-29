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
            right: vec![BarItem::Clock,BarItem::Battery],
        }
    }

    pub(super) fn items(&self) -> Vec<BarItem> {
        let mut items = Vec::new();

        items.extend(self.left.iter().copied());
        items.extend(self.center.iter().copied());
        items.extend(self.right.iter().copied());

        items
    }

    pub(super) fn contains_duplicates(&self) -> bool {
        let items = self.items();

        items.iter().enumerate().any(|(index, item)| {
            items[index + 1..].contains(item)
        })
    }
}
