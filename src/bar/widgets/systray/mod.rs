pub(crate) mod model;
pub(crate) mod service;

pub(crate) const ID: &str = "systray";

use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{BoxExt, GestureSingleExt, PopoverExt, WidgetExt};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use wayle_systray::adapters::gtk4::Adapter;

use self::model::SystrayItemSummary;
use crate::bar::BarMsg;
use crate::bar::state::{BarItemState, SystrayState};
use crate::bar::widget::{
    BarContext, BarWidget, BarWidgetRuntime, WidgetAction, WidgetEvent, WidgetInstance,
};
use crate::shell::ShellMsg;

const ICON_EXTENSIONS: [&str; 3] = ["png", "svg", "xpm"];

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct SystrayConfig {
    icon_size: i32,
}

impl Default for SystrayConfig {
    fn default() -> Self {
        Self {
            icon_size: 16,
        }
    }
}

impl SystrayConfig {
    fn icon_size(&self) -> i32 {
        self.icon_size.max(1)
    }
}

#[derive(Default)]
struct SystrayIconCache {
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

struct SystrayRuntime {
    root: gtk::Box,
    sender: relm4::Sender<BarMsg>,
    items: HashMap<String, SystrayItemRuntime>,
    icon_cache: SystrayIconCache,
    icon_size: i32,
}

impl SystrayRuntime {
    fn reconcile_items(&mut self, items: &[SystrayItemSummary]) {
        let mut desired_keys = HashSet::new();

        for item in items {
            let key = logical_item_key(item);
            if !desired_keys.insert(key.clone()) {
                tracing::info!(
                    id = %item.id,
                    bus_name = %item.bus_name,
                    key = %key,
                    "Skipping duplicate systray item"
                );
                continue;
            }

            if let Some(runtime) = self.items.get_mut(&key) {
                runtime.update(item, &mut self.icon_cache, self.icon_size);
            } else {
                let runtime = SystrayItemRuntime::new(&self.sender, item, &mut self.icon_cache, self.icon_size);
                self.root.append(&runtime.root);
                self.items.insert(key, runtime);
            }
        }

        self.items.retain(|key, runtime| {
            if desired_keys.contains(key) {
                true
            } else {
                self.root.remove(&runtime.root);
                false
            }
        });
    }
}

struct SystrayItemRuntime {
    root: gtk::Box,
    status_class: Option<String>,
    last_item: Option<SystrayItemSummary>,
}

impl SystrayItemRuntime {
    fn new(
        sender: &relm4::Sender<BarMsg>,
        item: &SystrayItemSummary,
        icon_cache: &mut SystrayIconCache,
        icon_size: i32,
    ) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.add_css_class("bar-item");
        root.add_css_class("systray");

        attach_click_handler(root.upcast_ref(), sender, item);

        let mut runtime = Self {
            root,
            status_class: None,
            last_item: None,
        };
        runtime.update(item, icon_cache, icon_size);
        runtime
    }

    fn update(&mut self, item: &SystrayItemSummary, icon_cache: &mut SystrayIconCache, icon_size: i32) {
        if self.last_item.as_ref() == Some(item) {
            return;
        }

        self.last_item = Some(item.clone());

        while let Some(child) = self.root.first_child() {
            self.root.remove(&child);
        }

        if let Some(status_class) = self.status_class.take() {
            self.root.remove_css_class(&status_class);
        }

        let status_class = format!("systray-{}", item.status.to_lowercase());
        self.root.add_css_class(&status_class);
        self.status_class = Some(status_class);

        self.root
            .set_tooltip_text(systray_tooltip(item).as_deref());

        let child = systray_item_content(item, icon_cache, icon_size);
        self.root.append(&child);
    }
}

impl BarWidgetRuntime for SystrayRuntime {
    fn root(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, _context: &BarContext) {
        let BarItemState::Systray(SystrayState::Ready(items)) = state else {
            return;
        };

        self.reconcile_items(items);
    }
}
pub(crate) struct SystrayWidget;

impl BarWidget for SystrayWidget {
    fn id(&self) -> &'static str {
        ID
    }

    fn build(
        &self,
        instance: &WidgetInstance,
        sender: &relm4::Sender<BarMsg>,
    ) -> Box<dyn BarWidgetRuntime> {
        let config = instance.config_as::<SystrayConfig>();
        let icon_size = config.icon_size();
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        Box::new(SystrayRuntime {
            root,
            sender: sender.clone(),
            items: HashMap::new(),
            icon_cache: SystrayIconCache::default(),
            icon_size,
        })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Systray(SystrayState::Ready(Vec::new())))
    }

    fn start(&self, sender: Sender<ShellMsg>) -> Option<relm4::JoinHandle<()>> {
        Some(service::start(sender))
    }
}

fn attach_click_handler(
    widget: &gtk::Widget,
    sender: &relm4::Sender<BarMsg>,
    item: &SystrayItemSummary,
) {
    let click = gtk::GestureClick::new();
    click.set_button(0);

    let sender = sender.clone();
    let bus_name = item.bus_name.clone();
    let parent = widget.clone();

    click.connect_released(move |gesture, _n_press, x, y| {
        let button = gesture.current_button();

        if button == 3 {
            let parent = parent.clone();
            let bus_name = bus_name.clone();

            gtk::glib::idle_add_local_once(move || {
                show_menu(&parent, &bus_name);
            });

            return;
        }

        let _ = sender.send(BarMsg::WidgetEvent(WidgetEvent {
            widget_id: ID,
            action: WidgetAction::Clicked {
                item_id: bus_name.clone(),
                button: gesture.current_button(),
                x: x as i32,
                y: y as i32,
            },
        }));
    });

    widget.add_controller(click)
}

fn image_from_pixmap(pixmap: &wayle_systray::types::item::IconPixmap, icon_size: i32) -> gtk::Image {
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

fn show_menu(parent: &gtk::Widget, bus_name: &str) {
    let Some(item) = service::item_by_bus_name(bus_name) else {
        tracing::warn!("Systray item disappeared before menu could be shown: {bus_name}");
        return;
    };

    let popover = Adapter::build_popover(item.as_ref());
    popover.set_parent(parent);
    popover.popup();
}

fn systray_item_content(
    item: &SystrayItemSummary,
    icon_cache: &mut SystrayIconCache,
    icon_size: i32,
) -> gtk::Widget {
    if let (Some(icon_theme_path), Some(icon_name)) = (&item.icon_theme_path, &item.icon_name) {
        icon_cache.add_theme_path(icon_theme_path);

        if let Some(path) = icon_cache.resolve_icon_path(icon_theme_path, icon_name) {
            let image = gtk::Image::from_file(path);
            image.set_pixel_size(16);
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

fn logical_item_key(item: &SystrayItemSummary) -> String {
    if !item.id.is_empty() {
        return format!("id:{}", item.id);
    }

    format!("bus:{}", item.bus_name)
}

fn systray_tooltip(item: &SystrayItemSummary) -> Option<String> {
    for value in [
        item.tooltip_title.as_str(),
        item.tooltip_description.as_str(),
        item.title.as_str(),
        item.id.as_str(),
    ] {
        if !value.trim().is_empty() {
            return Some(value.trim().to_string());
        }
    }

    None
}