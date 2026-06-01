use wayle_systray::core::item::TrayItem;

#[derive(Clone, Debug)]
pub(crate) struct SystrayItemSummary {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) icon_name: Option<String>,
    pub(crate) status: String,
    pub(crate) bus_name: String,
}

impl SystrayItemSummary {
    pub(crate) fn from_wayle_item(item: &TrayItem) -> Self {
        tracing::debug!(
            id = %item.id.get(),
            title = %item.title.get(),
            icon_name = ?item.icon_name.get(),
            icon_pixmap_count = item.icon_pixmap.get().len(),
            icon_theme_path = ?item.icon_theme_path.get(),
            "systray item icon data"
        );
        Self {
            id: item.id.get(),
            title: item.title.get(),
            icon_name: item.icon_name.get(),
            status: item.status.get().to_string(),
            bus_name: item.bus_name.get(),
        }
    }
}