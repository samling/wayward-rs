use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(super) struct SystrayConfig {
    icon_size: i32,
}

impl Default for SystrayConfig {
    fn default() -> Self {
        Self { icon_size: 16 }
    }
}

impl SystrayConfig {
    pub(super) fn icon_size(&self) -> i32 {
        self.icon_size.max(1)
    }
}
