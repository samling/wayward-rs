mod config;
mod icon;
mod interaction;
pub(crate) mod model;
pub(crate) mod service;

pub(crate) const ID: &str = "systray";

use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{BoxExt, WidgetExt};
use std::collections::{HashMap, HashSet};

use self::config::SystrayConfig;
use self::icon::{SystrayIconCache, systray_item_content};
use self::interaction::attach_click_handler;
use self::model::SystrayItemSummary;
use crate::bar::BarMsg;
use crate::bar::state::{BarItemState, SystrayState};
use crate::bar::widget::{BarContext, BarWidget, BarWidgetRuntime, WidgetInstance};
use crate::shell::ShellMsg;

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
                let runtime = SystrayItemRuntime::new(
                    &self.sender,
                    item,
                    &mut self.icon_cache,
                    self.icon_size,
                );
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

    fn update(
        &mut self,
        item: &SystrayItemSummary,
        icon_cache: &mut SystrayIconCache,
        icon_size: i32,
    ) {
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

        self.root.set_tooltip_text(item.tooltip_text().as_deref());

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
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);

        Box::new(SystrayRuntime {
            root,
            sender: sender.clone(),
            items: HashMap::new(),
            icon_cache: SystrayIconCache::default(),
            icon_size: config.icon_size(),
        })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Systray(SystrayState::Ready(Vec::new())))
    }

    fn start(
        &self,
        sender: Sender<ShellMsg>,
        services: &crate::services::ShellServices,
    ) -> Option<relm4::JoinHandle<()>> {
        Some(service::start(sender, services.systray.clone()))
    }
}

fn logical_item_key(item: &SystrayItemSummary) -> String {
    if !item.id.is_empty() {
        return format!("id:{}", item.id);
    }

    format!("bus:{}", item.bus_name)
}
