use relm4::gtk;
use relm4::gtk::prelude::*;
use std::sync::Arc;
use wayle_systray::SystemTrayService;
use wayle_systray::adapters::gtk4::Adapter;

use super::view_model::SystrayItemSummary;
use super::{ID, service};
use crate::bar::BarMsg;
use crate::bar::widget::{WidgetAction, WidgetEvent};

pub(super) fn attach_click_handler(
    widget: &gtk::Widget,
    sender: &relm4::Sender<BarMsg>,
    item: &SystrayItemSummary,
    service: Option<Arc<SystemTrayService>>,
) {
    let click = gtk::GestureClick::new();
    click.set_button(0);

    let sender = sender.clone();
    let bus_name = item.bus_name.clone();
    let parent = widget.clone();
    let service = service.clone();

    click.connect_released(move |gesture, _n_press, x, y| {
        let button = gesture.current_button();

        if button == 3 {
            let parent = parent.clone();
            let bus_name = bus_name.clone();
            let service = service.clone();

            gtk::glib::idle_add_local_once(move || {
                show_menu(&parent, &bus_name, service.as_deref());
            });

            return;
        }

        let _ = sender.send(BarMsg::WidgetEvent(WidgetEvent {
            widget_id: ID,
            action: WidgetAction::Clicked {
                item_id: bus_name.clone(),
                button,
                x: x as i32,
                y: y as i32,
            },
        }));
    });

    widget.add_controller(click)
}

fn show_menu(parent: &gtk::Widget, bus_name: &str, service: Option<&SystemTrayService>) {
    let Some(service) = service else {
        tracing::warn!("Ignoring systray menu before service is ready");
        return;
    };

    let Some(item) = service::item_by_bus_name(service, bus_name) else {
        tracing::warn!("Systray item disappeared before menu could be shown: {bus_name}");
        return;
    };

    let popover = Adapter::build_popover(item.as_ref());
    popover.add_css_class("systray-menu");
    popover.set_parent(parent);
    popover.popup();
}
