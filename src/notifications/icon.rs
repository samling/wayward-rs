use std::path::Path;

use relm4::gtk;

use super::model::NotificationToast;

pub(crate) fn set_notification_icon(image: &gtk::Image, notification: &NotificationToast) {
    for candidate in [
        notification.image_path.as_deref(),
        Some(notification.app_icon.as_str()),
    ]
    .into_iter()
    .flatten()
    {
        if set_icon_candidate(image, candidate) {
            return;
        }
    }

    image.set_icon_name(Some("dialog-information-symbolic"));
}

fn set_icon_candidate(image: &gtk::Image, candidate: &str) -> bool {
    let candidate = candidate.trim();

    if candidate.is_empty() {
        return false;
    }

    if let Some(path) = candidate.strip_prefix("file://") {
        image.set_from_file(Some(path));
        return true;
    }

    if Path::new(candidate).is_file() {
        image.set_from_file(Some(candidate));
        return true;
    }

    image.set_icon_name(Some(candidate));
    true
}