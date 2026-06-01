use relm4::gtk;

use super::Bar;
use super::state::BarItemState;
use crate::shell::ShellMsg;

#[derive(Clone)]
pub(crate) struct WidgetInstance {
    pub(crate) id: String,
    pub(crate) widget_type: String,
    pub(crate) widget: &'static dyn BarWidget,
    pub(crate) config: toml::value::Table,
}

impl std::fmt::Debug for WidgetInstance {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WidgetInstance")
            .field("id", &self.id)
            .field("widget_type", &self.widget_type)
            .finish()
    }
}
pub(crate) trait BarWidget: Sync {
    fn id(&self) -> &'static str;

    fn render(&self, bar: &Bar, instance: &WidgetInstance, container: &gtk::Box);

    fn initial_state(&self) -> Option<BarItemState> {
        None
    }

    fn start(&self, _sender: relm4::Sender<ShellMsg>) -> Option<relm4::JoinHandle<()>> {
        None
    }
}
