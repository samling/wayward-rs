pub(crate) mod audio;
pub(crate) mod brightness;
pub(crate) mod window;

#[derive(Clone, Debug)]
pub(crate) enum OsdEvent {
    Brightness { percent: f64 },
    Volume { percent: f64, muted: bool },
}

impl OsdEvent {
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Brightness { percent } => format!("Brightness: {:.0}%", percent),
            Self::Volume { percent, muted } if *muted => {
                format!("Volume muted {:.0}%", percent)
            }
            Self::Volume { percent, muted: _ } => format!("Volume: {:.0}%", percent),
        }
    }

    pub(crate) fn class_name(&self) -> &'static str {
        match self {
            Self::Brightness { .. } => "brightness",
            Self::Volume { muted, .. } if *muted => "muted",
            Self::Volume { .. } => "volume",
        }
    }

    pub(crate) fn icon(&self) -> &'static str {
        match self {
            Self::Brightness { .. } => "󰃠",
            Self::Volume { muted: true, .. } => "󰝟",
            Self::Volume { .. } => "󰕾",
        }
    }

    pub(crate) fn percent(&self) -> f64 {
        match self {
            Self::Brightness { percent } | Self::Volume { percent, .. } => *percent,
        }
    }
}
