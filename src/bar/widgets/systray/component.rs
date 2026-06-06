use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wayle_systray::SystemTrayService;

use crate::bar::BarMsg;
use crate::bar::layout::BarEdge;

use super::icon::{SystrayIconCache, systray_item_content};
use super::interaction::attach_click_handler;
use super::view_model::SystrayItemSummary;

pub(super) struct SystrayComponent {
    content: gtk::Box,
    sender: relm4::Sender<BarMsg>,
    items: HashMap<String, SystrayItemRuntime>,
    icon_cache: SystrayIconCache,
    icon_size: i32,
    orientation: gtk::Orientation,
    service: Option<Arc<SystemTrayService>>,
}

pub(super) struct SystrayInit {
    pub(super) edge: BarEdge,
    pub(super) icon_size: i32,
    pub(super) instance_class: Option<String>,
    pub(super) sender: relm4::Sender<BarMsg>,
    pub(super) service: Option<Arc<SystemTrayService>>,
}

#[derive(Debug)]
pub(super) enum SystrayInput {
    SetItems(Vec<SystrayItemSummary>),
}

pub(super) struct SystrayWidgets;

impl SimpleComponent for SystrayComponent {
    type Init = SystrayInit;
    type Input = SystrayInput;
    type Output = ();
    type Root = gtk::Box;
    type Widgets = SystrayWidgets;

    fn init_root() -> Self::Root {
        gtk::Box::new(gtk::Orientation::Horizontal, 0)
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let orientation = init.edge.orientation();

        root.set_orientation(orientation);
        crate::bar::style::add_bar_item_classes(&root, super::ID, init.instance_class.as_deref());

        let content = gtk::Box::new(orientation, 4);
        content.add_css_class("bar-item-content");
        content.add_css_class("systray-content");
        root.append(&content);

        let model = Self {
            content,
            sender: init.sender,
            items: HashMap::new(),
            icon_cache: SystrayIconCache::default(),
            icon_size: init.icon_size,
            orientation,
            service: init.service,
        };

        ComponentParts {
            model,
            widgets: SystrayWidgets,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SystrayInput::SetItems(items) => {
                self.reconcile_items(&items);
            }
        }
    }
}

impl SystrayComponent {
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
                    self.orientation,
                    self.service.clone(),
                );
                self.content.append(&runtime.root);
                self.items.insert(key, runtime);
            }
        }

        self.items.retain(|key, runtime| {
            if desired_keys.contains(key) {
                true
            } else {
                self.content.remove(&runtime.root);
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
        orientation: gtk::Orientation,
        service: Option<Arc<SystemTrayService>>,
    ) -> Self {
        let root = gtk::Box::new(orientation, 0);
        root.add_css_class("systray-item");

        attach_click_handler(root.upcast_ref(), sender, item, service);

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

fn logical_item_key(item: &SystrayItemSummary) -> String {
    if !item.id.is_empty() {
        return format!("id:{}", item.id);
    }

    format!("bus:{}", item.bus_name)
}
