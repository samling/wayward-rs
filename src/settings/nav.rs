#[allow(dead_code)]
pub(crate) struct NavGroup {
    pub(crate) title: &'static str,
    pub(crate) items: &'static [NavItem],
}

#[allow(dead_code)]
pub(crate) struct NavItem {
    pub(crate) key: &'static str,
    pub(crate) title: &'static str,
    pub(crate) content: NavContent,
}

#[allow(dead_code)]
pub(crate) enum NavContent {
    StyleSection(&'static str),
    Widget {
        section: &'static str,
        config_key: &'static str,
    },
    BarLayout,
}

#[allow(dead_code)]
pub(crate) const DEFAULT_ITEM: &str = "palette";

#[allow(dead_code)]
pub(crate) fn nav() -> &'static [NavGroup] {
    use NavContent::{BarLayout, StyleSection, Widget};

    &[
        NavGroup {
            title: "Appearance",
            items: &[
                NavItem { key: "palette", title: "Palette", content: StyleSection("Palette") },
                NavItem { key: "bar", title: "Bar", content: StyleSection("Bar") },
                NavItem {
                    key: "settings-window",
                    title: "Settings window",
                    content: StyleSection("Settings window"),
                },
            ],
        },
        NavGroup {
            title: "Widgets",
            items: &[
                NavItem {
                    key: "workspaces",
                    title: "Workspaces",
                    content: Widget { section: "Workspaces", config_key: "workspaces" },
                },
                NavItem {
                    key: "battery",
                    title: "Battery",
                    content: Widget { section: "Battery", config_key: "battery" },
                },
                NavItem {
                    key: "systray",
                    title: "Systray",
                    content: Widget { section: "Systray", config_key: "systray" },
                },
                NavItem {
                    key: "clock",
                    title: "Clock",
                    content: Widget { section: "Clock", config_key: "clock" },
                },
                NavItem {
                    key: "brightness",
                    title: "Brightness",
                    content: Widget { section: "Brightness", config_key: "brightness" },
                },
                NavItem {
                    key: "volume",
                    title: "Volume",
                    content: Widget { section: "Volume", config_key: "volume" },
                },
                NavItem {
                    key: "updates",
                    title: "Updates",
                    content: Widget { section: "Updates", config_key: "updates" },
                },
                NavItem {
                    key: "notifications",
                    title: "Notifications",
                    content: Widget { section: "Notifications", config_key: "notifications" },
                },
                NavItem {
                    key: "notification-cards",
                    title: "Notification cards",
                    content: Widget { section: "Notification cards", config_key: "notifications" },
                },
                NavItem {
                    key: "osd",
                    title: "OSD",
                    content: Widget { section: "OSD", config_key: "osd" },
                },
                NavItem {
                    key: "action-menu",
                    title: "Action menu",
                    content: Widget { section: "Action menu", config_key: "action-menu" },
                },
            ],
        },
        NavGroup {
            title: "Layout",
            items: &[NavItem { key: "bars", title: "Bars", content: BarLayout }],
        },
    ]
}

#[allow(dead_code)]
pub(crate) fn find_item(key: &str) -> Option<&'static NavItem> {
    nav()
        .iter()
        .flat_map(|group| group.items.iter())
        .find(|item| item.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::page::{build_page, SettingsConfig};
    use std::collections::HashSet;

    #[test]
    fn nav_item_keys_are_unique() {
        let mut seen = HashSet::new();
        for group in nav() {
            for item in group.items {
                assert!(seen.insert(item.key), "duplicate nav key: {}", item.key);
            }
        }
    }

    #[test]
    fn default_item_exists() {
        assert!(find_item(DEFAULT_ITEM).is_some());
    }

    #[test]
    fn build_page_for_appearance_item_has_single_section() {
        let item = find_item("palette").unwrap();
        let config = SettingsConfig {
            style: crate::config::StyleConfig::default(),
            widgets: std::collections::BTreeMap::new(),
            bars: Vec::new(),
            available_monitors: Vec::new(),
        };
        let page = build_page(item, &config).unwrap();
        assert_eq!(page.title, "Palette");
        assert_eq!(page.sections.len(), 1);
    }

    #[test]
    fn build_page_for_bar_layout_is_none() {
        let item = find_item("bars").unwrap();
        let config = SettingsConfig {
            style: crate::config::StyleConfig::default(),
            widgets: std::collections::BTreeMap::new(),
            bars: Vec::new(),
            available_monitors: Vec::new(),
        };
        assert!(build_page(item, &config).is_none());
    }
}
