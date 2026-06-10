use relm4::factory::FactoryComponent;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

use crate::notifications::model::NotificationToast;

const BODY_PREVIEW_LINES: usize = 4;

pub(super) struct NotificationRow {
    notification: NotificationToast,
}

#[derive(Debug)]
pub(super) enum NotificationRowInput {
    InvokeDefault,
    InvokeAction(String),
    Dismiss,
}

#[derive(Debug)]
pub(super) enum NotificationRowOutput {
    InvokeDefault(u32),
    InvokeAction { id: u32, action_id: String },
    Dismiss(u32),
}

#[relm4::factory(pub(super))]
impl FactoryComponent for NotificationRow {
    type Init = NotificationToast;
    type Input = NotificationRowInput;
    type Output = NotificationRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[name = "root"]
        gtk::Box {
            add_css_class: "notification-list-row",
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,
            set_width_request: 320,

            #[name = "icon"]
            gtk::Image {
                add_css_class: "notification-list-icon",
                set_valign: gtk::Align::Start,
            },

            gtk::Box {
                add_css_class: "notification-list-text",
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 2,
                set_hexpand: true,

                add_controller = gtk::GestureClick {
                    set_button: 1,

                    connect_released[sender] => move|_, _, _, _,| {
                        sender.input(NotificationRowInput::InvokeDefault);
                    }
                },

                gtk::Label {
                    add_css_class: "notification-list-app-name",
                    set_halign: gtk::Align::Start,
                    set_xalign: 0.0,
                    set_max_width_chars: 34,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,

                    #[watch]
                    set_text: &self.notification.app_name,
                },

                gtk::Label {
                    add_css_class: "notification-list-summary",
                    set_halign: gtk::Align::Start,
                    set_xalign: 0.0,
                    set_wrap: true,
                    set_wrap_mode: gtk::pango::WrapMode::WordChar,
                    set_natural_wrap_mode: gtk::NaturalWrapMode::Word,
                    set_lines: 2,
                    set_width_chars: 34,
                    set_max_width_chars: 34,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,

                    #[watch]
                    set_text: &self.notification.summary,
                },

                gtk::Label {
                    add_css_class: "notification-list-body",
                    set_halign: gtk::Align::Start,
                    set_xalign: 0.0,
                    set_wrap: true,
                    set_wrap_mode: gtk::pango::WrapMode::WordChar,
                    set_natural_wrap_mode: gtk::NaturalWrapMode::Word,
                    set_lines: 3,
                    set_width_chars: 34,
                    set_max_width_chars: 34,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,

                    #[watch]
                    set_visible: self.has_body(),

                    #[watch]
                    set_text: &self.body_preview_text(),
                },

                #[name = "actions"]
                gtk::Box {
                    add_css_class: "notification-list-actions",
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,
                    set_visible: !self.notification.visible_actions().is_empty(),
                }
            },

            gtk::Button {
                add_css_class: "notification-list-dismiss",
                add_css_class: "flat",
                set_valign: gtk::Align::Start,
                set_icon_name: "window-close-symbolic",

                connect_clicked[sender] => move |_| {
                    sender.input(NotificationRowInput::Dismiss);
                }
            },
        }
    }

    fn pre_view() {
        self.sync_imperative_view(
            &widgets.root,
            &widgets.icon,
            &widgets.actions,
            sender.clone(),
        );
    }

    fn init_model(
        notification: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { notification }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        self.sync_imperative_view(&widgets.root, &widgets.icon, &widgets.actions, sender);
        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            NotificationRowInput::InvokeDefault => {
                let _ = sender.output(NotificationRowOutput::InvokeDefault(self.notification.id));
            }
            NotificationRowInput::InvokeAction(action_id) => {
                let _ = sender.output(NotificationRowOutput::InvokeAction {
                    id: self.notification.id,
                    action_id,
                });
            }
            NotificationRowInput::Dismiss => {
                let _ = sender.output(NotificationRowOutput::Dismiss(self.notification.id));
            }
        }
    }
}

impl NotificationRow {
    fn sync_imperative_view(
        &self,
        root: &gtk::Box,
        icon: &gtk::Image,
        actions_box: &gtk::Box,
        sender: FactorySender<Self>,
    ) {
        for class_name in ["low", "normal", "critical"] {
            root.remove_css_class(class_name);
        }

        root.add_css_class(self.notification.urgency_class());

        crate::notifications::icon::set_notification_icon(icon, &self.notification);

        while let Some(child) = actions_box.first_child() {
            actions_box.remove(&child);
        }

        let actions = self.notification.visible_actions();
        actions_box.set_visible(!actions.is_empty());

        for action in actions {
            let button = gtk::Button::with_label(&action.label);
            button.add_css_class("notification-list-action");

            let sender = sender.clone();
            button.connect_clicked(move |_| {
                sender.input(NotificationRowInput::InvokeAction(action.id.clone()));
            });

            actions_box.append(&button);
        }
    }

    pub(super) fn set_notification(&mut self, notification: NotificationToast) {
        self.notification = notification;
    }

    fn body_preview_text(&self) -> String {
        self.notification
            .body
            .as_deref()
            .map(compact_preview_text)
            .unwrap_or_default()
    }

    fn has_body(&self) -> bool {
        self.notification
            .body
            .as_ref()
            .is_some_and(|body| !body.trim().is_empty())
    }
}

fn compact_preview_text(value: &str) -> String {
    let mut lines = value.lines().take(BODY_PREVIEW_LINES).collect::<Vec<_>>();

    if value.lines().count() > BODY_PREVIEW_LINES {
        if let Some(last_line) = lines.last_mut() {
            *last_line = "...";
        }
    }
    lines.join("\n")
}
