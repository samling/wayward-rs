pub(crate) mod model;
pub(crate) mod service;

use relm4::Sender;
use relm4::gtk;
use relm4::gtk::glib::object::Cast;
use relm4::gtk::prelude::{BoxExt, GestureSingleExt, PopoverExt, WidgetExt};
use wayle_systray::adapters::gtk4::Adapter;

use self::model::{SystrayEvent, SystrayItemSummary};
use crate::bar::BarMsg;
use crate::bar::state::{BarItemState, SystrayState};
use crate::bar::widget::{BarContext, BarWidget, BarWidgetRuntime, WidgetEvent, WidgetInstance};
use crate::shell::ShellMsg;

struct SystrayRuntime {
    root: gtk::Box,
    sender: relm4::Sender<BarMsg>,
}

impl BarWidgetRuntime for SystrayRuntime {
    fn root(&self) -> gtk::Widget {
        self.root.clone().upcast()
    }

    fn update(&mut self, state: &BarItemState, _context: &BarContext) {
        let BarItemState::Systray(SystrayState::Ready(items)) = state else {
            return;
        };

        render_items(&self.root, &self.sender, items);
    }
}
pub(crate) struct SystrayWidget;

impl BarWidget for SystrayWidget {
    fn id(&self) -> &'static str {
        "systray"
    }

    fn build(
        &self,
        _instance: &WidgetInstance,
        sender: &relm4::Sender<BarMsg>,
    ) -> Box<dyn BarWidgetRuntime> {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);

        Box::new(SystrayRuntime {
            root,
            sender: sender.clone(),
        })
    }

    fn initial_state(&self) -> Option<BarItemState> {
        Some(BarItemState::Systray(SystrayState::Ready(Vec::new())))
    }

    fn start(&self, sender: Sender<ShellMsg>) -> Option<relm4::JoinHandle<()>> {
        Some(service::start(sender))
    }
}

fn render_items(
    container: &gtk::Box,
    sender: &relm4::Sender<BarMsg>,
    items: &[SystrayItemSummary],
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

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
}
