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
pub(super) enum SunsetrState {
    NotRunning,
    Automatic(SunsetrStatus),
    Paused(SunsetrStatus),
    Custom(SunsetrStatus),
    Unknown(String),
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

    pub(super) fn detail_text(&self) -> String {
        match self {
            Self::NotRunning => "sunsetr is not running".to_string(),
            Self::Automatic(status) | Self::Paused(status) | Self::Custom(status) => {
                status.detail_text()
            }
            Self::Unknown(error) => error.clone(),
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
}

impl SunsetrStatus {
    fn detail_text(&self) -> String {
        let mut parts = Vec::new();

        if self.active_preset != "default" {
            parts.push(format!("Preset {}", self.active_preset));
        }

        for value in [
            self.current_period.as_deref(),
            self.temperature.as_deref(),
            self.gamma.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            parts.push(value.to_string());
        }

        if let Some(next_period) = &self.next_period {
            parts.push(format!("Next {next_period}"));
        }

        parts.join(" | ")
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
mod tests {
    use super::*;

    #[test]
    fn parses_automatic_status() {
        let status = parse_status(
            r#"
 Active preset: default
Current period: Day 󰖨
         State: stable
   Temperature: 6500K
         Gamma: 100.0%
   Next period: 19:18:46 (in 7h6m)
"#,
        )
        .unwrap();

        assert_eq!(status.active_preset, "default");
        assert_eq!(status.current_period.as_deref(), Some("Day 󰖨"));
        assert_eq!(status.state.as_deref(), Some("stable"));
        assert_eq!(status.temperature.as_deref(), Some("6500K"));
        assert_eq!(status.gamma.as_deref(), Some("100.0%"));
        assert_eq!(status.next_period.as_deref(), Some("19:18:46 (in 7h6m)"));
    }

    #[test]
    fn parses_static_status_without_next_period() {
        let status = parse_status(
            r#"
 Active preset: day
Current period: Static 󰋙
         State: static
   Temperature: 6500K
         Gamma: 100.0%
"#,
        )
        .unwrap();

        assert_eq!(status.active_preset, "day");
        assert_eq!(status.current_period.as_deref(), Some("Static 󰋙"));
        assert_eq!(status.next_period, None);
    }

    #[test]
    fn classifies_default_preset_as_automatic() {
        let config = SunsetrConfig::default();
        let status = SunsetrStatus {
            active_preset: "default".to_string(),
            current_period: Some("Day 󰖨".to_string()),
            state: Some("stable".to_string()),
            temperature: Some("6500K".to_string()),
            gamma: Some("100.0%".to_string()),
            next_period: None,
        };

        assert!(matches!(
            SunsetrState::from_status(status, &config),
            SunsetrState::Automatic(_)
        ));
    }

    #[test]
    fn classifies_paused_preset_as_paused() {
        let config = SunsetrConfig::default();
        let status = SunsetrStatus {
            active_preset: "day".to_string(),
            current_period: Some("Static 󰋙".to_string()),
            state: Some("static".to_string()),
            temperature: Some("6500K".to_string()),
            gamma: Some("100.0%".to_string()),
            next_period: None,
        };

        assert!(matches!(
            SunsetrState::from_status(status, &config),
            SunsetrState::Paused(_)
        ));
    }
}
