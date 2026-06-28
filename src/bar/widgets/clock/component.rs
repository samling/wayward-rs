use chrono::{DateTime, Local};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, SimpleComponent,
};

use crate::bar::layout::BarEdge;
use crate::bar::menu_button;
use crate::bar::widget::BarRegion;

use super::dropdown::{ClockDropdown, ClockDropdownInit, ClockDropdownInput};

pub(super) struct ClockComponent {
    edge: BarEdge,
    region: BarRegion,
    format: String,
    time: DateTime<Local>,
    label_text: String,
    dropdown: Controller<ClockDropdown>,
}

pub(super) struct ClockInit {
    pub(super) edge: BarEdge,
    pub(super) region: BarRegion,
    pub(super) format: String,
    pub(super) instance_class: Option<String>,
}

#[derive(Debug)]
pub(super) enum ClockInput {
    SetPlacement { edge: BarEdge, region: BarRegion },
    SetTime(DateTime<Local>),
}

pub(super) struct ClockWidgets {
    content: gtk::Box,
    label: gtk::Label,
}

fn format_time(time: &DateTime<Local>, format: &str) -> String {
    use std::fmt::Write;

    let mut buffer = String::new();
    if write!(buffer, "{}", time.format(format)).is_ok() {
        buffer
    } else {
        time.format("%H:%M").to_string()
    }
}

impl SimpleComponent for ClockComponent {
    type Init = ClockInit;
    type Input = ClockInput;
    type Output = ();
    type Root = gtk::MenuButton;
    type Widgets = ClockWidgets;

    fn init_root() -> Self::Root {
        gtk::MenuButton::new()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let time = Local::now();
        let label_text = format_time(&time, &init.format);
        let dropdown = ClockDropdown::builder()
            .launch(ClockDropdownInit {
                date: time.date_naive(),
                edge: init.edge,
                region: init.region,
            })
            .detach();

        let model = Self {
            edge: init.edge,
            region: init.region,
            format: init.format,
            time,
            label_text,
            dropdown,
        };

        let content = menu_button::content_box(model.edge.orientation(), 0, "clock-content");

        let label = gtk::Label::new(Some(&model.label_text));
        label.add_css_class("clock-label");
        crate::bar::style::configure_bar_label(&label);
        content.append(&label);

        menu_button::configure_root(&root, "clock", init.instance_class.as_deref());
        menu_button::attach_content(&root, &content);
        menu_button::attach_popover(&root, model.dropdown.widget());

        let widgets = ClockWidgets { content, label };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ClockInput::SetPlacement { edge, region } => {
                self.edge = edge;
                self.region = region;
                self.dropdown
                    .emit(ClockDropdownInput::SetPlacement { edge, region });
            }
            ClockInput::SetTime(time) => {
                let previous_date = self.time.date_naive();

                self.label_text = format_time(&time, &self.format);
                self.time = time;

                let date = self.time.date_naive();
                if date != previous_date {
                    self.dropdown.emit(ClockDropdownInput::SetDate(date));
                }
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.content.set_orientation(self.edge.orientation());
        widgets.label.set_text(&self.label_text);
    }
}
