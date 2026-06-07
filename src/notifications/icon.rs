use std::path::Path;

use relm4::gtk;

use super::model::NotificationToast;

pub(crate) fn set_notification_icon(image: &gtk::Image, notification: &NotificationToast) {
    if let Some(path) = notification.image_path.as_deref() {
        image.set_from_file(Some(path));
        return;
    }

    if Path::new(&notification.app_icon).is_file() {
        image.set_from_file(Some(&notification.app_icon));
        return;
    }

    image.set_icon_name(Some(&notification.app_icon));
}