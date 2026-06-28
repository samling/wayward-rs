use super::config::SunsetrConfig;
use relm4::tokio::process::Command as TokioCommand;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SunsetrStatus {
    pub(super) active_preset: String,
    pub(super) current_period: Option<String>,
    pub(super) state: Option<String>,
    pub(super) temperature: Option<String>,
    pub(super) gamma: Option<String>,
    pub(super) next_period: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SunsetrDetailRow {
    pub(super) label: &'static str,
    pub(super) value: String,
}

impl SunsetrDetailRow {
    pub(super) fn new(label: &'static str, value: impl Into<String>) -> Self {
        Self {
            label,
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum SunsetrDetails {
    Rows(Vec<SunsetrDetailRow>),
    Message(String),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum SunsetrState {
    NotRunning,
    Automatic(SunsetrStatus),
    Paused(SunsetrStatus),
    Custom(SunsetrStatus),
    Unknown(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PendingSunsetrAction {
    paused: bool,
    remaining_stale_checks: u8,
}

impl PendingSunsetrAction {
    pub(super) fn new(paused: bool) -> Self {
        Self {
            paused,
            remaining_stale_checks: 4,
        }
    }

    pub(super) fn accepts_status(&mut self, state: &SunsetrState) -> bool {
        if state.matches_paused(self.paused) {
            return true;
        }

        if self.remaining_stale_checks == 0 {
            return true;
        }

        self.remaining_stale_checks -= 1;
        false
    }
}

impl SunsetrState {
    pub(super) fn from_status(status: SunsetrStatus, config: &SunsetrConfig) -> Self {
        if status.active_preset == config.automatic_preset {
            Self::Automatic(status)
        } else if status.active_preset == config.paused_preset {
            Self::Paused(status)
        } else {
            Self::Custom(status)
        }
    }

    pub(super) fn status_text(&self) -> &'static str {
        match self {
            Self::NotRunning => "Not running",
            Self::Automatic(_) => "Automatic",
            Self::Paused(_) => "Paused",
            Self::Custom(_) => "Custom",
            Self::Unknown(_) => "Unknown",
        }
    }

    pub(super) fn details(&self) -> SunsetrDetails {
        match self {
            Self::NotRunning => SunsetrDetails::Message("sunsetr is not running".to_string()),
            Self::Automatic(status) | Self::Paused(status) | Self::Custom(status) => {
                SunsetrDetails::Rows(status.detail_rows())
            }
            Self::Unknown(error) => SunsetrDetails::Message(error.clone()),
        }
    }

    pub(super) fn action_label(&self) -> Option<&'static str> {
        match self {
            Self::Automatic(_) => Some("Pause"),
            Self::Paused(_) | Self::Custom(_) => Some("Resume"),
            Self::NotRunning | Self::Unknown(_) => None,
        }
    }

    pub(super) fn action_paused(&self) -> Option<bool> {
        match self {
            Self::Automatic(_) => Some(true),
            Self::Paused(_) | Self::Custom(_) => Some(false),
            Self::NotRunning | Self::Unknown(_) => None,
        }
    }

    pub(super) fn with_action_applied(&self, paused: bool, config: &SunsetrConfig) -> Self {
        let preset = if paused {
            &config.paused_preset
        } else {
            &config.automatic_preset
        };

        match self {
            Self::Automatic(status) | Self::Paused(status) | Self::Custom(status) => {
                let mut status = status.clone();
                status.active_preset = preset.clone();
                Self::from_status(status, config)
            }
            Self::NotRunning | Self::Unknown(_) => Self::Unknown("Updating sunsetr".to_string()),
        }
    }

    fn matches_paused(&self, paused: bool) -> bool {
        matches!(
            (paused, self),
            (true, Self::Paused(_)) | (false, Self::Automatic(_))
        )
    }
}

impl SunsetrStatus {
    fn detail_rows(&self) -> Vec<SunsetrDetailRow> {
        let mut rows = vec![SunsetrDetailRow::new("Preset", &self.active_preset)];

        for (label, value) in [
            ("Period", self.current_period.as_deref()),
            ("Temperature", self.temperature.as_deref()),
            ("Gamma", self.gamma.as_deref()),
            ("Next", self.next_period.as_deref()),
        ]
        .into_iter()
        .filter_map(|(label, value)| value.map(|value| (label, value)))
        {
            rows.push(SunsetrDetailRow::new(label, value));
        }

        rows
    }
}

pub(super) fn parse_status(input: &str) -> Option<SunsetrStatus> {
    let mut active_preset = None;
    let mut current_period = None;
    let mut state = None;
    let mut temperature = None;
    let mut gamma = None;
    let mut next_period = None;

    for line in input.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };

        let key = key.trim();
        let value = value.trim();

        if value.is_empty() {
            continue;
        }

        match key {
            "Active preset" => active_preset = Some(value.to_string()),
            "Current period" => current_period = Some(value.to_string()),
            "State" => state = Some(value.to_string()),
            "Temperature" => temperature = Some(value.to_string()),
            "Gamma" => gamma = Some(value.to_string()),
            "Next period" => next_period = Some(value.to_string()),
            _ => {}
        }
    }

    Some(SunsetrStatus {
        active_preset: active_preset?,
        current_period,
        state,
        temperature,
        gamma,
        next_period,
    })
}

pub(super) async fn current_state(config: SunsetrConfig) -> SunsetrState {
    if !is_running().await {
        return SunsetrState::NotRunning;
    }

    match TokioCommand::new("sunsetr").arg("status").output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            parse_status(&stdout)
                .map(|status| SunsetrState::from_status(status, &config))
                .unwrap_or_else(|| {
                    SunsetrState::Unknown("Could not parse sunsetr status".to_string())
                })
        }
        Err(error) => SunsetrState::Unknown(format!("Failed to run sunsetr status: {error}")),
    }
}

async fn is_running() -> bool {
    // Use output() so pgrep's matched PID is captured, not inherited to our stdout.
    TokioCommand::new("pgrep")
        .arg("-x")
        .arg("sunsetr")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "sunsetr_test.rs"]
mod tests;
