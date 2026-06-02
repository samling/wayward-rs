use wayle_systray::core::item::TrayItem;
use wayle_systray::types::item::IconPixmap;

#[derive(Clone, PartialEq)]
pub(crate) struct SystrayItemSummary {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) icon_name: Option<String>,
    pub(crate) icon_pixmaps: Vec<IconPixmap>,
    pub(crate) icon_theme_path: Option<String>,
    pub(crate) tooltip_title: String,
    pub(crate) tooltip_description: String,
    pub(crate) status: String,
    pub(crate) bus_name: String,
}

impl std::fmt::Debug for SystrayItemSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystrayItemSummary")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("icon_name", &self.icon_name)
            .field("icon_pixmap_count", &self.icon_pixmaps.len())
            .field("icon_theme_path", &self.icon_theme_path)
            .field("status", &self.status)
            .field("bus_name", &self.bus_name)
            .finish()
    }
}

impl SystrayItemSummary {
    pub(crate) fn from_wayle_item(item: &TrayItem) -> Self {
        let tooltip = item.tooltip.get();
        Self {
            id: item.id.get(),
            title: item.title.get(),
            icon_name: item.icon_name.get(),
            icon_pixmaps: item.icon_pixmap.get(),
            icon_theme_path: item.icon_theme_path.get(),
            tooltip_title: tooltip.title,
            tooltip_description: tooltip.description,
            status: item.status.get().to_string(),
            bus_name: item.bus_name.get(),
        }
    }
}
