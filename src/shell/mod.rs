mod bars;
mod hot_reload;
mod monitors;
mod notification_overlays;
mod osd_windows;

use gtk::prelude::*;
use relm4::gtk;
use relm4::prelude::*;

use crate::{
    bar,
    config::{AppConfig, ConfigChanges},
};

pub(crate) struct Shell {
    bars: Vec<bars::RunningBar>,
    config: AppConfig,
    item_states: Vec<bar::state::BarItemState>,
    focused_monitor_connector: Option<String>,
    popup_notifications: Vec<crate::notifications::model::NotificationToast>,
    notification_windows: Vec<notification_overlays::RunningNotificationWindow>,
    osd_windows: Vec<osd_windows::RunningOsd>,
    settings_window: Option<Controller<crate::settings::window::SettingsWindow>>,
    services: crate::services::ShellServices,
    style: Option<crate::style::StyleHandle>,
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
    InvokeNotificationAction { id: u32, action_id: String },
    InvokeNotificationDefaultAction(u32),
    DismissNotificationPopup(u32),
    OpenSettings,
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
            settings_window: None,
            services,
            style: style.clone(),
        };

        model.apply_generated_style();
        model.reconcile_bars(sender.input_sender().clone());
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
                let changes = ConfigChanges::between(&self.config, &config);

                if !changes.has_changes() {
                    return;
                }

                tracing::info!(?changes, "Config changed");

                self.config = config;

                self.sync_settings_window();

                if changes.bars_changed || changes.widgets_changed {
                    self.reconcile_bars(_sender.input_sender().clone());
                }

                if changes.notifications_changed {
                    self.show_notifications();
                }

                if changes.style_changed {
                    self.apply_generated_style();
                }
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
                self.reconcile_bars(_sender.input_sender().clone());
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
            ShellMsg::OpenSettings => {
                self.open_settings_window();
            }
        }
    }
}

impl Shell {
    fn apply_generated_style(&self) {
        let Some(style) = &self.style else {
            return;
        };

        let css = crate::style::generated_style_config(&self.config.style);

        if style.set_generated_css(css) {
            for running_bar in &self.bars {
                let _ = running_bar
                    .controller
                    .sender()
                    .send(bar::BarMsg::StyleChanged);
            }
        }
    }

    fn open_settings_window(&mut self) {
        if let Some(settings_window) = &self.settings_window {
            settings_window.widget().present();
            return;
        }

        let settings_window = crate::settings::window::SettingsWindow::builder()
            .launch(crate::settings::window::SettingsConfig::from(&self.config))
            .detach();

        if let Some(settings_window) = &self.settings_window {
            settings_window
                .sender()
                .send(crate::settings::window::SettingsInput::SetConfig(
                    crate::settings::window::SettingsConfig::from(&self.config),
                ))
                .ok();

            settings_window.widget().present();
            return;
        }

        settings_window.widget().present();
        self.settings_window = Some(settings_window);
    }

    fn sync_settings_window(&self) {
        let Some(settings_window) = &self.settings_window else {
            return;
        };

        settings_window
            .sender()
            .send(crate::settings::window::SettingsInput::SetConfig(
                crate::settings::window::SettingsConfig::from(&self.config),
            ))
            .ok();
    }
}
