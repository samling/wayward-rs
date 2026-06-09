use relm4::{gtk::{self, prelude::{GtkWindowExt, OrientableExt, WidgetExt}}, prelude::*};

pub(crate) struct SettingsWindow;

#[relm4::component(pub(crate))]
impl SimpleComponent for SettingsWindow {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Wayward Settings"),
            set_default_size: (900, 650),
            set_hide_on_close: true,
            add_css_class: "settings-window",

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_width_request: 220,
                    add_css_class: "settings-sidebar",

                    gtk::Label {
                        set_label: "Settings",
                        set_halign: gtk::Align::Start,
                        add_css_class: "settings-sidebar-title",
                    },

                    gtk::Button {
                        add_css_class: "settings-sidebar-item",
                        set_sensitive: false,

                        gtk::Label {
                            set_label: "Notifications",
                            set_halign: gtk::Align::Start,
                        },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_vexpand: true,
                    add_css_class: "settings-page",

                    gtk::Label {
                        set_label: "Notifications",
                        set_halign: gtk::Align::Start,
                        add_css_class: "settings-page-description",
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}