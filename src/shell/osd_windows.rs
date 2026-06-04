use super::{Shell, monitors};

pub(super) struct RunningOsd {
    pub(super) connector: String,
    pub(super) window: crate::osd::window::OsdWindow,
}

impl Shell {
    pub(super) fn reconcile_osd_windows(&mut self) {
        let monitors = monitors::available();

        self.osd_windows.retain(|osd| {
            monitors.iter().any(|monitor| {
                monitors::connector(monitor).as_deref() == Some(osd.connector.as_str())
            })
        });

        for monitor in monitors {
            let Some(connector) = monitors::connector(&monitor) else {
                continue;
            };

            if self
                .osd_windows
                .iter()
                .any(|osd| osd.connector == connector)
            {
                continue;
            }

            self.osd_windows.push(RunningOsd {
                connector,
                window: crate::osd::window::OsdWindow::new(&monitor),
            });
        }
    }

    pub(super) fn show_osd(&mut self, event: &crate::osd::OsdEvent) {
        let Some(focused_connector) = self.focused_monitor_connector.as_deref() else {
            tracing::info!("Skipping OSD event because no focused monitor is known");
            return;
        };

        let Some(osd) = self
            .osd_windows
            .iter()
            .find(|osd| osd.connector == focused_connector)
        else {
            tracing::info!("Skipping OSD event because OSD window is unavailable");
            return;
        };

        osd.window.show_event(event);
    }
}
