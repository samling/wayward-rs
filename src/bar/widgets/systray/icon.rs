use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::model::SystrayItemSummary;

const ICON_EXTENSIONS: [&str; 3] = ["png", "svg", "xpm"];

#[derive(Default)]
pub(super) struct SystrayIconCache {
    theme_paths: HashSet<String>,
    icon_paths: HashMap<(String, String), Option<PathBuf>>,
}

impl SystrayIconCache {
    fn add_theme_path(&mut self, path: &str) {
        if !self.theme_paths.insert(path.to_string()) {
            return;
        }

        let Some(display) = gtk::gdk::Display::default() else {
            return;
        };

        let icon_theme = gtk::IconTheme::for_display(&display);
        icon_theme.add_search_path(path);
    }

    fn resolve_icon_path(&mut self, theme_path: &str, icon_name: &str) -> Option<PathBuf> {
        let key = (theme_path.to_string(), icon_name.to_string());

        if let Some(path) = self.icon_paths.get(&key) {
            return path.clone();
        }

        let resolved = ICON_EXTENSIONS
            .iter()
            .map(|extension| Path::new(theme_path).join(format!("{icon_name}.{extension}")))
            .find(|path| path.is_file());

        self.icon_paths.insert(key, resolved.clone());
        resolved
    }
}

pub(super) fn systray_item_content(
    item: &SystrayItemSummary,
    icon_cache: &mut SystrayIconCache,
    icon_size: i32,
) -> gtk::Widget {
    if let (Some(icon_theme_path), Some(icon_name)) = (&item.icon_theme_path, &item.icon_name) {
        icon_cache.add_theme_path(icon_theme_path);

        if let Some(path) = icon_cache.resolve_icon_path(icon_theme_path, icon_name) {
            let image = gtk::Image::from_file(path);
            image.set_pixel_size(icon_size);
            return image.upcast();
        }
    }

    if let Some(icon_name) = &item.icon_name {
        if let Some(image) = image_from_icon_file(icon_name, icon_size) {
            return image.upcast();
        }
    }

    if let Some(icon_name) = &item.icon_name
        && let Some(image) = image_from_icon_name(icon_name, icon_size)
    {
        return image.upcast();
    }

    if let Some(pixmap) = item.icon_pixmaps.first() {
        return image_from_pixmap(pixmap, icon_size).upcast();
    }

    let text = if !item.title.is_empty() {
        item.title.as_str()
    } else {
        item.id.as_str()
    };

    gtk::Label::new(Some(text)).upcast()
}

fn image_from_pixmap(
    pixmap: &wayle_systray::types::item::IconPixmap,
    icon_size: i32,
) -> gtk::Image {
    let bytes = gtk::glib::Bytes::from_owned(pixmap.data.clone());
    let texture = gtk::gdk::MemoryTexture::new(
        pixmap.width,
        pixmap.height,
        gtk::gdk::MemoryFormat::A8r8g8b8,
        &bytes,
        pixmap.width as usize * 4,
    );

    let image = gtk::Image::from_paintable(Some(&texture));
    image.set_pixel_size(icon_size);
    image
}

fn image_from_icon_name(icon_name: &str, icon_size: i32) -> Option<gtk::Image> {
    let display = gtk::gdk::Display::default()?;
    let icon_theme = gtk::IconTheme::for_display(&display);

    if !icon_theme.has_icon(icon_name) {
        return None;
    }

    let image = gtk::Image::from_icon_name(icon_name);
    image.set_pixel_size(icon_size);
    Some(image)
}

fn image_from_icon_file(path: &str, icon_size: i32) -> Option<gtk::Image> {
    let texture = gtk::gdk::Texture::from_filename(path).ok()?;
    let image = gtk::Image::from_paintable(Some(&texture));
    image.set_pixel_size(icon_size);
    Some(image)
}
