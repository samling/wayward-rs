use relm4::gtk;

use super::BarMsg;
use super::state::BarItemState;
use crate::bar::widgets::systray::model::SystrayEvent;
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

#[derive(Clone, Debug)]
pub(crate) enum WidgetEvent {
    Systray(SystrayEvent),
}

pub(crate) trait BarWidget: Sync {
    fn id(&self) -> &'static str;

    fn build(
        &self,
        instance: &WidgetInstance,
        sender: &relm4::Sender<BarMsg>,
    ) -> Box<dyn BarWidgetRuntime>;

    fn initial_state(&self) -> Option<BarItemState> {
        None
    }

    fn start(&self, _sender: relm4::Sender<ShellMsg>) -> Option<relm4::JoinHandle<()>> {
        None
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BarContext {
    pub(crate) monitor_connector: Option<String>,
}

pub(crate) trait BarWidgetRuntime {
    fn root(&self) -> gtk::Widget;

    fn update(&mut self, state: &BarItemState, context: &BarContext);
}
