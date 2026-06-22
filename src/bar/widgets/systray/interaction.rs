use relm4::gtk;
use relm4::gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wayle_systray::SystemTrayService;
use wayle_systray::adapters::gtk4::Adapter;
use wayle_systray::types::menu::MenuItem;

use super::view_model::SystrayItemSummary;
use super::{ID, service};
use crate::bar::BarMsg;
use crate::bar::widget::{SystrayAction, WidgetAction, WidgetEvent};

#[derive(Clone)]
pub(super) struct SystrayMenuCache(Rc<RefCell<SystrayMenuState>>);

#[derive(Default)]
struct SystrayMenuState {
    popover: Option<gtk::PopoverMenu>,
    menu: Option<MenuItem>,
    action_group: Option<gtk::gio::SimpleActionGroup>,
}

pub(super) fn new_menu_cache() -> SystrayMenuCache {
    SystrayMenuCache(Rc::new(RefCell::new(SystrayMenuState::default())))
}

pub(super) fn attach_click_handler(
    widget: &gtk::Widget,
    sender: &relm4::Sender<BarMsg>,
    item: &SystrayItemSummary,
    service: Option<Arc<SystemTrayService>>,
    menu_cache: SystrayMenuCache,
) {
    let click = gtk::GestureClick::new();
    click.set_button(0);

    let sender = sender.clone();
    let bus_name = item.bus_name.clone();
    let parent = widget.downgrade();
    let service = service.clone();
    let menu_cache = menu_cache.clone();

    click.connect_released(move |gesture, _n_press, x, y| {
        let button = gesture.current_button();

        if button == 3 {
            let parent = parent.clone();
            let bus_name = bus_name.clone();
            let service = service.clone();
            let menu_cache = menu_cache.clone();

            gtk::glib::idle_add_local_once(move || {
                let Some(parent) = parent.upgrade() else {
                    return;
                };
                show_menu(&parent, &bus_name, service.as_deref(), &menu_cache);
            });

            return;
        }

        let _ = sender.send(BarMsg::WidgetEvent(WidgetEvent {
            widget_id: ID,
            action: WidgetAction::Systray(SystrayAction::Clicked {
                item_id: bus_name.clone(),
                button,
                x: x as i32,
                y: y as i32,
            }),
        }));
    });

    widget.add_controller(click)
}

pub(super) fn unparent_cached_menu(menu_cache: &SystrayMenuCache) {
    let Some(popover) = menu_cache.0.borrow_mut().popover.take() else {
        return;
    };

    popover.popdown();
    popover.set_menu_model(None::<&gtk::gio::Menu>);
    popover.insert_action_group("app", None::<&gtk::gio::SimpleActionGroup>);

    if popover.parent().is_some() {
        popover.unparent();
    }

    menu_cache.0.borrow_mut().menu = None;
    menu_cache.0.borrow_mut().action_group = None;
}

fn show_menu(
    parent: &gtk::Widget,
    bus_name: &str,
    service: Option<&SystemTrayService>,
    menu_cache: &SystrayMenuCache,
) {
    let Some(service) = service else {
        tracing::warn!("Ignoring systray menu before service is ready");
        return;
    };

    let Some(item) = service::item_by_bus_name(service, bus_name) else {
        tracing::warn!("Systray item disappeared before menu could be shown: {bus_name}");
        return;
    };

    let current_menu = item.menu.get();
    let popover = cached_popover(parent, menu_cache);

    if menu_cache.0.borrow().menu != current_menu {
        let model = Adapter::build_model(item.as_ref());
        popover.set_menu_model(Some(&model.menu));
        popover.insert_action_group("app", Some(&model.actions));

        let mut cache = menu_cache.0.borrow_mut();
        cache.menu = current_menu;
        cache.action_group = Some(model.actions);
    }

    popover.popup();
}

fn cached_popover(parent: &gtk::Widget, menu_cache: &SystrayMenuCache) -> gtk::PopoverMenu {
    let mut menu_cache = menu_cache.0.borrow_mut();

    if let Some(popover) = menu_cache.popover.as_ref() {
        return popover.clone();
    }

    let popover = gtk::PopoverMenu::from_model(None::<&gtk::gio::Menu>);
    popover.add_css_class("systray-menu");
    popover.set_parent(parent);

    menu_cache.popover = Some(popover.clone());

    popover
}
