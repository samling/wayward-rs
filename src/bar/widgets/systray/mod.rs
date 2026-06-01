pub(crate) mod model;
pub(crate) mod service;

use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{GestureSingleExt, BoxExt, WidgetExt, PopoverExt};
use wayle_systray::adapters::gtk4::Adapter;

use self::model::{SystrayEvent, SystrayItemSummary};
use crate::bar::state::{BarItemState, SystrayState};
use crate::bar::widget::{BarWidget, WidgetEvent, WidgetInstance};
use crate::bar::{Bar, BarMsg};
use crate::shell::ShellMsg;

pub(crate) struct SystrayWidget;

impl BarWidget for SystrayWidget {
    fn id(&self) -> &'static str {
        "systray"
    }

    fn render(
        &self,
        bar: &Bar,
        _instance: &WidgetInstance,
        container: &gtk::Box,
        sender: &relm4::Sender<BarMsg>,
    ) {
        let Some(state) = bar.item_states().iter().find_map(|state| match state {
            BarItemState::Systray(state) => Some(state),
            _ => None,
        }) else {
            return;
        };

        let SystrayState::Ready(items) = state else {
            return;
        };

        tracing::info!("Rendering systray with {} item(s)", items.len());
        for item in items {
            let child: gtk::Widget = if let Some(icon_name) = &item.icon_name {
                let image = gtk::Image::from_icon_name(icon_name);
                image.set_pixel_size(16);
                image.upcast()
            } else {
                let text = if !item.title.is_empty() {
                    item.title.as_str()
                } else {
                    item.id.as_str()
                };

                gtk::Label::new(Some(text)).upcast()
            };
            child.add_css_class("bar-item");
            child.add_css_class("systray");
            child.add_css_class(&format!("systray-{}", item.status.to_lowercase()));
            child.set_tooltip_text(Some(&item.title));

            attach_click_handler(&child, sender, item);

            container.append(&child);
        }
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

        if button == 1 || button == 3 {
            let parent = parent.clone();
            let bus_name = bus_name.clone();

            gtk::glib::idle_add_local_once(move || {
                show_menu(&parent, &bus_name);
            });

            return;
        }

        let _ = sender.send(BarMsg::WidgetEvent(WidgetEvent::Systray(
            SystrayEvent::Clicked {
                bus_name: bus_name.clone(),
                button: gesture.current_button(),
                x: x as i32,
                y: y as i32,
            },
        )));
    });

    widget.add_controller(click)
}

fn show_menu(parent: &gtk::Widget, bus_name: &str) {
    let Some(item) = service::item_by_bus_name(bus_name) else {
        tracing::warn!("Systram item disappeared before menu could be shown: {bus_name}");
        return;
    };

    let popover = Adapter::build_popover(item.as_ref());
    popover.set_parent(parent);
    popover.popup();

    popover.connect_closed(|_| {
        tracing::info!("Systray popover closed");
    });
}