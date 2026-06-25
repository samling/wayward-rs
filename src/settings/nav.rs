pub(crate) struct NavGroup {
    pub(crate) title: &'static str,
    pub(crate) items: &'static [NavItem],
}

pub(crate) struct NavItem {
    pub(crate) key: &'static str,
    pub(crate) title: &'static str,
    pub(crate) content: NavContent,
}

pub(crate) enum NavContent {
    StyleSection(&'static str),
    Widget {
        section: &'static str,
        config_key: &'static str,
    },
    BarLayout,
}

pub(crate) const DEFAULT_ITEM: &str = "palette";

pub(crate) fn nav() -> &'static [NavGroup] {
    use NavContent::{BarLayout, StyleSection, Widget};

    &[
        NavGroup {
            title: "Appearance",
            items: &[
                NavItem {
                    key: "palette",
                    title: "Palette",
                    content: StyleSection("Palette"),
                },
                NavItem {
                    key: "bar",
                    title: "Bar",
                    content: StyleSection("Bar"),
                },
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
                    content: Widget {
                        section: "Workspaces",
                        config_key: "workspaces",
                    },
                },
                NavItem {
                    key: "battery",
                    title: "Battery",
                    content: Widget {
                        section: "Battery",
                        config_key: "battery",
                    },
                },
                NavItem {
                    key: "systray",
                    title: "Systray",
                    content: Widget {
                        section: "Systray",
                        config_key: "systray",
                    },
                },
                NavItem {
                    key: "clock",
                    title: "Clock",
                    content: Widget {
                        section: "Clock",
                        config_key: "clock",
                    },
                },
                NavItem {
                    key: "brightness",
                    title: "Brightness",
                    content: Widget {
                        section: "Brightness",
                        config_key: "brightness",
                    },
                },
                NavItem {
                    key: "volume",
                    title: "Volume",
                    content: Widget {
                        section: "Volume",
                        config_key: "volume",
                    },
                },
                NavItem {
                    key: "updates",
                    title: "Updates",
                    content: Widget {
                        section: "Updates",
                        config_key: "updates",
                    },
                },
                NavItem {
                    key: "notifications",
                    title: "Notifications",
                    content: Widget {
                        section: "Notifications",
                        config_key: "notifications",
                    },
                },
                NavItem {
                    key: "notification-cards",
                    title: "Notification cards",
                    content: Widget {
                        section: "Notification cards",
                        config_key: "notifications",
                    },
                },
                NavItem {
                    key: "osd",
                    title: "OSD",
                    content: Widget {
                        section: "OSD",
                        config_key: "osd",
                    },
                },
                NavItem {
                    key: "action-menu",
                    title: "Action menu",
                    content: Widget {
                        section: "Action menu",
                        config_key: "action-menu",
                    },
                },
            ],
        },
        NavGroup {
            title: "Layout",
            items: &[NavItem {
                key: "bars",
                title: "Bars",
                content: BarLayout,
            }],
        },
    ]
}

pub(crate) fn find_item(key: &str) -> Option<&'static NavItem> {
    nav()
        .iter()
        .flat_map(|group| group.items.iter())
        .find(|item| item.key == key)
}

#[cfg(test)]
#[path = "nav_test.rs"]
mod tests;
