use std::sync::Arc;

use relm4::ComponentSender;
use wayle_audio::AudioService;
use wayle_battery::BatteryService;
use wayle_brightness::BrightnessService;
use wayle_niri::NiriService;
use wayle_systray::SystemTrayService;

use crate::shell::Shell;

#[derive(Clone, Default)]
pub(crate) struct ShellServices {
    pub(crate) audio: Option<Arc<AudioService>>,
    pub(crate) battery: Option<Arc<BatteryService>>,
    pub(crate) brightness: Option<Arc<BrightnessService>>,
    pub(crate) niri: Option<Arc<NiriService>>,
    pub(crate) systray: Option<Arc<SystemTrayService>>,
}

pub(crate) async fn init_shell_services() -> ShellServices {
    let audio = match AudioService::builder().build().await {
        Ok(service) => {
            tracing::info!("Audio service started");
            Some(service)
        }
        Err(error) => {
            tracing::error!("Failed to start audio service: {error}");
            None
        }
    };

    let battery = match BatteryService::new().await {
        Ok(service) => {
            tracing::info!("Battery service started");
            Some(Arc::new(service))
        }
        Err(error) => {
            tracing::error!("Failed to start battery service: {error}");
            None
        }
    };

    let brightness = match BrightnessService::new().await {
        Ok(Some(service)) => {
            tracing::info!("Brightness service started");
            Some(service)
        }
        Ok(None) => {
            tracing::info!("No brightness device found, brightness OSD disabled");
            None
        }
        Err(error) => {
            tracing::error!("Failed to start brightness service: {error}");
            None
        }
    };

    let niri = match NiriService::new().await {
        Ok(service) => {
            tracing::info!("Niri service started");
            Some(service)
        }
        Err(error) => {
            tracing::error!("Failed to start Niri service: {error}");
            None
        }
    };

    let systray = match SystemTrayService::new().await {
        Ok(service) => {
            tracing::info!("System tray service started");
            Some(service)
        }
        Err(error) => {
            tracing::error!("Failed to start system tray service: {error}");
            None
        }
    };

    ShellServices {
        audio,
        battery,
        brightness,
        niri,
        systray,
    }
}

pub(crate) fn initial_item_states() -> Vec<crate::bar::state::BarItemState> {
    crate::bar::registry::WIDGETS
        .iter()
        .filter_map(|widget| widget.initial_state())
        .collect()
}

pub(crate) fn start_all(sender: &ComponentSender<Shell>, services: &ShellServices) {
    let input_sender = sender.input_sender().clone();

    for widget in crate::bar::registry::WIDGETS {
        widget.start(input_sender.clone(), services);
    }

    crate::osd::audio::start(input_sender.clone(), services.audio.clone());
    crate::osd::brightness::start(input_sender, services.brightness.clone());
}
