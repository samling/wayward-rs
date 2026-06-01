use wayle_systray::core::item::TrayItem;
use wayle_systray::types::item::IconPixmap;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SystrayItemSummary {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) icon_name: Option<String>,
    pub(crate) icon_pixmaps: Vec<IconPixmap>,
    pub(crate) icon_theme_path: Option<String>,
    pub(crate) status: String,
    pub(crate) bus_name: String,
}

impl SystrayItemSummary {
    pub(crate) fn from_wayle_item(item: &TrayItem) -> Self {
        Self {
            id: item.id.get(),
            title: item.title.get(),
            icon_name: item.icon_name.get(),
            icon_pixmaps: item.icon_pixmap.get(),
            icon_theme_path: item.icon_theme_path.get(),
            status: item.status.get().to_string(),
            bus_name: item.bus_name.get(),
        }
    }
}
