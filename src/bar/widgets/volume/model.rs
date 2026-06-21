#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AudioDeviceSummary {
    pub(crate) key: u32,
    pub(crate) label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VolumeSnapshot {
    pub(crate) percent: f64,
    pub(crate) muted: bool,
    pub(crate) outputs: Vec<AudioDeviceSummary>,
    pub(crate) inputs: Vec<AudioDeviceSummary>,
    pub(crate) default_output: Option<u32>,
    pub(crate) default_input: Option<u32>,
}

impl VolumeSnapshot {
    pub(crate) fn display_percent(&self) -> f64 {
        if self.muted { 0.0 } else { self.percent }
    }

    pub(crate) fn percent_text(&self) -> String {
        format!("{:.0}%", self.display_percent())
    }

    pub(crate) fn icon_name(&self) -> &'static str {
        let percent = self.display_percent();

        if self.muted || percent <= 0.0 {
            "audio-volume-muted-symbolic"
        } else if percent < 34.0 {
            "audio-volume-low-symbolic"
        } else if percent < 67.0 {
            "audio-volume-medium-symbolic"
        } else {
            "audio-volume-high-symbolic"
        }
    }
}
