use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use super::view_model::SystrayItemSummary;

const ICON_EXTENSIONS: [&str; 3] = ["png", "svg", "xpm"];

thread_local! {
    static ICON_PAINTABLES: RefCell<HashMap<IconCacheKey, gtk::gdk::Paintable>> =
        RefCell::new(HashMap::new());
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum IconCacheKey {
    File {
        path: PathBuf,
    },
    ThemeName {
        name: String,
        size: i32,
    },
    Pixmap {
        width: i32,
        height: i32,
        digest: u64,
    },
}

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
            if let Some(image) = image_from_icon_path(path, icon_size) {
                return image.upcast();
            }
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

    let label = gtk::Label::new(Some(text));
    crate::bar::style::configure_bar_label(&label);
    label.upcast()
}

fn image_from_pixmap(
    pixmap: &wayle_systray::types::item::IconPixmap,
    icon_size: i32,
) -> gtk::Image {
    let key = IconCacheKey::Pixmap {
        width: pixmap.width,
        height: pixmap.height,
        digest: pixmap_digest(pixmap),
    };

    let paintable = cached_paintable(key, || {
        let bytes = gtk::glib::Bytes::from_owned(pixmap.data.clone());
        let texture = gtk::gdk::MemoryTexture::new(
            pixmap.width,
            pixmap.height,
            gtk::gdk::MemoryFormat::A8r8g8b8,
            &bytes,
            pixmap.width as usize * 4,
        );

        Some(texture.upcast())
    });

    let image = gtk::Image::from_paintable(paintable.as_ref());
    image.set_pixel_size(icon_size);
    image
}

fn image_from_icon_name(icon_name: &str, icon_size: i32) -> Option<gtk::Image> {
    let display = gtk::gdk::Display::default()?;
    let icon_theme = gtk::IconTheme::for_display(&display);

    if !icon_theme.has_icon(icon_name) {
        return None;
    }

    let key = IconCacheKey::ThemeName {
        name: icon_name.to_string(),
        size: icon_size,
    };

    let paintable = cached_paintable(key, || {
        let paintable = icon_theme.lookup_icon(
            icon_name,
            &[],
            icon_size,
            1,
            gtk::TextDirection::None,
            gtk::IconLookupFlags::empty(),
        );

        Some(paintable.upcast())
    })?;

    let image = gtk::Image::from_paintable(Some(&paintable));
    image.set_pixel_size(icon_size);
    Some(image)
}

fn image_from_icon_file(path: &str, icon_size: i32) -> Option<gtk::Image> {
    image_from_icon_path(PathBuf::from(path), icon_size)
}

fn image_from_icon_path(path: PathBuf, icon_size: i32) -> Option<gtk::Image> {
    let key = IconCacheKey::File { path: path.clone() };

    let paintable = cached_paintable(key, || {
        let texture = gtk::gdk::Texture::from_filename(&path).ok()?;
        Some(texture.upcast())
    })?;

    let image = gtk::Image::from_paintable(Some(&paintable));
    image.set_pixel_size(icon_size);
    Some(image)
}

fn cached_paintable(
    key: IconCacheKey,
    load: impl FnOnce() -> Option<gtk::gdk::Paintable>,
) -> Option<gtk::gdk::Paintable> {
    if let Some(paintable) = ICON_PAINTABLES.with(|cache| cache.borrow().get(&key).cloned()) {
        return Some(paintable);
    }

    let paintable = load()?;
    ICON_PAINTABLES.with(|cache| cache.borrow_mut().insert(key, paintable.clone()));

    Some(paintable)
}

fn pixmap_digest(pixmap: &wayle_systray::types::item::IconPixmap) -> u64 {
    let mut hasher = DefaultHasher::new();
    pixmap.data.hash(&mut hasher);
    hasher.finish()
}
