use relm4::gtk;
use serde::de::DeserializeOwned;

use super::BarMsg;
use super::state::BarItemState;
use crate::services::ShellServices;
use crate::shell::ShellMsg;

#[derive(Clone)]
pub(crate) struct WidgetInstance {
    pub(crate) id: String,
    pub(crate) widget_type: String,
    pub(crate) instance: Option<String>,
    pub(crate) widget: &'static dyn BarWidget,
    pub(crate) config: toml::value::Table,
}

impl WidgetInstance {
    pub(crate) fn config_as<T>(&self) -> T
    where
        T: DeserializeOwned + Default,
    {
        let value = toml::Value::Table(self.config.clone());

        match value.try_into() {
            Ok(config) => config,
            Err(error) => {
                tracing::error!(
                    widget_id = %self.id,
                    widget_type = %self.widget_type,
                    "Failed to parse widget config: {error}"
                );
                T::default()
            }
        }
    }

    pub(crate) fn instance_css_class(&self) -> Option<String> {
        self.instance
            .as_ref()
            .map(|instance| format!("instance-{}-{}", self.widget_type, instance))
    }
}

impl PartialEq for WidgetInstance {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.widget_type == other.widget_type
            && self.instance == other.instance
            && self.config == other.config
    }
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
pub(crate) struct WidgetEvent {
    pub(crate) widget_id: &'static str,
    pub(crate) action: WidgetAction,
}

#[derive(Clone, Debug)]
pub(crate) enum WidgetAction {
    ActionMenu(ActionMenuAction),
    Brightness(BrightnessAction),
    Notifications(NotificationAction),
    Systray(SystrayAction),
    Updates(UpdatesAction),
    Volume(VolumeAction),
    Workspaces(WorkspaceAction),
}

#[derive(Clone, Debug)]
pub(crate) enum ActionMenuAction {
    Run { command: ActionMenuCommand },
    OpenSettings,
}

#[derive(Clone, Debug)]
pub(crate) enum BrightnessAction {
    SetBrightness { percent: f64 },
    RunBlueLightCommand { command: String },
}

#[derive(Clone, Debug)]
pub(crate) enum NotificationAction {
    InvokeDefault { id: u32 },
    InvokeAction { id: u32, action_id: String },
    Dismiss { id: u32 },
    DismissAll,
}

#[derive(Clone, Debug)]
pub(crate) enum SystrayAction {
    Clicked {
        item_id: String,
        button: u32,
        x: i32,
        y: i32,
    },
}

#[derive(Clone, Debug)]
pub(crate) enum UpdatesAction {
    Refresh,
}

#[derive(Clone, Debug)]
pub(crate) enum VolumeAction {
    SetOutputVolume { percent: f64 },
    ToggleOutputMute,
    SetDefaultOutput { key: u32 },
    SetDefaultInput { key: u32 },
}

#[derive(Clone, Debug)]
pub(crate) enum WorkspaceAction {
    Clicked { item_id: String, button: u32 },
}

#[derive(Clone, Debug)]
pub(crate) struct ActionMenuCommand {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BarRegion {
    Start,
    Center,
    End,
}

pub(crate) trait BarWidget: Sync {
    fn id(&self) -> &'static str;

    fn config_table_keys(&self) -> &'static [&'static str] {
        &[]
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        context: &WidgetBuildContext<'_>,
    ) -> Box<dyn BarWidgetRuntime>;

    fn initial_state(&self) -> Option<BarItemState> {
        None
    }

    fn handle_event(&self, event: WidgetEvent, _services: &ShellServices) {
        tracing::warn!(
            widget_id = %event.widget_id,
            widget_type = %self.id(),
            "Widget does not handle events"
        );
    }

    fn start(
        &self,
        _sender: relm4::Sender<ShellMsg>,
        _services: &ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        None
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BarContext {
    pub(crate) monitor_connector: Option<String>,
    pub(crate) edge: crate::bar::layout::BarEdge,
    pub(crate) region: BarRegion,
}

pub(crate) struct WidgetBuildContext<'a> {
    pub(crate) sender: &'a relm4::Sender<BarMsg>,
    pub(crate) services: &'a ShellServices,
    pub(crate) bar: &'a BarContext,
}

pub(crate) trait BarWidgetRuntime {
    fn root(&self) -> gtk::Widget;

    fn set_context(&mut self, _context: &BarContext) {}

    fn update(&mut self, state: &BarItemState, context: &BarContext);
}
