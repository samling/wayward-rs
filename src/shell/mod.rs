mod bars;
mod hot_reload;
mod monitors;
mod notification_overlays;
mod osd_windows;

use gtk::prelude::*;
use relm4::gtk;
use relm4::prelude::*;

use crate::{bar, config::AppConfig};

pub(crate) struct Shell {
    bars: Vec<bars::RunningBar>,
    config: AppConfig,
    item_states: Vec<bar::state::BarItemState>,
    focused_monitor_connector: Option<String>,
    popup_notifications: Vec<crate::notifications::model::NotificationToast>,
    notification_windows: Vec<notification_overlays::RunningNotificationWindow>,
    osd_windows: Vec<osd_windows::RunningOsd>,
    services: crate::services::ShellServices,
}

pub(crate) struct ShellInit {
    pub(crate) services: crate::services::ShellServices,
    pub(crate) style: Option<crate::style::StyleHandle>,
}

#[derive(Debug)]
pub(crate) enum ShellMsg {
    ConfigChanged(AppConfig),
    StyleChanged,
    MonitorsChanged,
    ReconcileMonitors,
    ItemStateChanged(bar::state::BarItemState),
    OsdChanged(crate::osd::OsdEvent),
    PopupNotificationsChanged(Vec<crate::notifications::model::NotificationToast>),
    DismissNotificationPopup(u32),
    InvokeNotificationAction { id: u32, action_id: String },
    InvokeNotificationDefaultAction(u32),
}

#[relm4::component(pub(crate))]
impl SimpleComponent for Shell {
    type Init = ShellInit;
    type Input = ShellMsg;
    type Output = ();

    view! {
        gtk::Window {
            set_visible: false,
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ShellInit { services, style } = init;
        let config = AppConfig::load();

        let mut model = Shell {
            bars: Vec::new(),
            config,
            item_states: crate::services::initial_item_states(),
            focused_monitor_connector: None,
            popup_notifications: Vec::new(),
            notification_windows: Vec::new(),
            osd_windows: Vec::new(),
            services,
        };

        model.reconcile_bars();
        model.reconcile_osd_windows();
        model.reconcile_notification_windows(&sender);

        Self::start_config_hot_reload(&sender);
        Self::start_monitor_watch(&sender);
        crate::services::start_all(&sender, &model.services);

        if let Some(style) = style {
            let input_sender = sender.input_sender().clone();

            crate::style::start_hot_reload(style, move || {
                let _ = input_sender.send(ShellMsg::StyleChanged);
            });
        }

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ShellMsg::ConfigChanged(config) => {
                self.config = config;
                self.reconcile_bars();
            }
            ShellMsg::StyleChanged => {
                for running_bar in &self.bars {
                    let _ = running_bar
                        .controller
                        .sender()
                        .send(bar::BarMsg::StyleChanged);
                }
            }
            ShellMsg::MonitorsChanged => {
                tracing::info!("Monitors changed");

                let input_sender = _sender.input_sender().clone();

                gtk::glib::timeout_add_once(std::time::Duration::from_millis(500), move || {
                    let _ = input_sender.send(ShellMsg::ReconcileMonitors);
                });
            }
            ShellMsg::ReconcileMonitors => {
                self.reconcile_bars();
                self.reconcile_osd_windows();
                self.reconcile_notification_windows(&_sender);

                if monitors::has_without_connector() {
                    let input_sender = _sender.input_sender().clone();

                    gtk::glib::timeout_add_once(std::time::Duration::from_millis(500), move || {
                        let _ = input_sender.send(ShellMsg::ReconcileMonitors);
                    });
                }
            }
            ShellMsg::ItemStateChanged(state) => {
                self.update_focused_monitor(&state);

                self.item_states
                    .retain(|existing_state| !existing_state.same_widget_as(&state));

                self.item_states.push(state.clone());

                for running_bar in &self.bars {
                    let _ = running_bar
                        .controller
                        .sender()
                        .send(bar::BarMsg::ItemStateChanged(state.clone()));
                }

                self.show_notifications();
            }
            ShellMsg::OsdChanged(event) => {
                self.show_osd(&event);
            }
            ShellMsg::PopupNotificationsChanged(notifications) => {
                self.popup_notifications = notifications;
                self.show_notifications();
            }
            ShellMsg::DismissNotificationPopup(id) => {
                self.dismiss_notification_popup(id);
            }
            ShellMsg::InvokeNotificationAction { id, action_id } => {
                self.invoke_notification_action(id, action_id);
            }
            ShellMsg::InvokeNotificationDefaultAction(id) => {
                self.invoke_notification_default_action(id);
            }
        }
    }
}
