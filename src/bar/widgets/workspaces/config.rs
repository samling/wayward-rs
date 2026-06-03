use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum WorkspaceIndicatorEffect {
    None,
    Slide,
    Ease,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct WorkspacesConfig {
    pub(super) indicator_effect: WorkspaceIndicatorEffect,
    pub(super) indicator_duration_ms: u64,
    pub(super) label_format: String,
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            indicator_effect: WorkspaceIndicatorEffect::Ease,
            indicator_duration_ms: 160,
            label_format: "%L".to_string(),
        }
    }
}

impl Default for WorkspaceIndicatorEffect {
    fn default() -> Self {
        Self::Ease
    }
}
