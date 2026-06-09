use std::path::Path;

use relm4::gtk;
use relm4::gtk::prelude::*;

use super::model::NotificationToast;

const FALLBACK_ICON_NAME: &str = "dialog-information-symbolic";

pub(crate) fn set_notification_icon(image: &gtk::Image, notification: &NotificationToast) {
    let app_icon =
        (notification.app_icon != FALLBACK_ICON_NAME).then_some(notification.app_icon.as_str());

    for candidate in [
        notification.image_path.as_deref(),
        app_icon,
        notification.desktop_entry.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if set_icon_candidate(image, candidate) {
            return;
        }
    }

    for candidate in app_name_icon_candidate(&notification.app_name) {
        if set_icon_candidate(image, &candidate) {
            return;
        }
    }

    image.set_icon_name(Some(FALLBACK_ICON_NAME));
}

fn set_icon_candidate(image: &gtk::Image, candidate: &str) -> bool {
    let candidate = candidate.trim();

    if candidate.is_empty() {
        return false;
    }

    if let Some(path) = candidate.strip_prefix("file://") {
        return set_file_icon(image, path);
    }

    if Path::new(candidate).is_file() {
        return set_file_icon(image, candidate);
    }

    if set_theme_icon(image, candidate) {
        return true;
    }

    candidate
        .strip_suffix(".desktop")
        .is_some_and(|icon_name| set_theme_icon(image, icon_name))
}

fn set_file_icon(image: &gtk::Image, path: &str) -> bool {
    let path = Path::new(path);

    if !path.is_file() {
        tracing::debug!(
            path = %path.display(),
            "Notification icon file candidate does not exist"
        );
        return false;
    }

    image.set_from_file(Some(path));
    true
}

fn set_theme_icon(image: &gtk::Image, icon_name: &str) -> bool {
    let theme = gtk::IconTheme::for_display(&image.display());

    if !theme.has_icon(icon_name) {
        return false;
    }

    image.set_icon_name(Some(icon_name));
    true
}

fn app_name_icon_candidate(app_name: &str) -> Vec<String> {
    let trimmed = app_name.trim();

    if trimmed.is_empty() {
        return Vec::new();
    }

    let lowercase = trimmed.to_lowercase();
    let kebab = lowercase.split_whitespace().collect::<Vec<_>>().join("-");

    if lowercase == kebab {
        vec![lowercase]
    } else {
        vec![lowercase, kebab]
    }
}
