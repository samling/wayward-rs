# Notification Toasts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add focused-monitor notification toast popups backed by `wayle-notification`.

**Architecture:** Notifications are shell-owned overlay windows, not bar widgets. `NotificationService::popups` feeds an owned UI model, and `Shell` targets the toast stack to the currently focused monitor.

**Tech Stack:** Rust 2024, Relm4 0.11, GTK4, gtk4-layer-shell, futures, chrono, wayle-notification.

---

## File Structure

- Modify `Cargo.toml`: add `wayle-notification = "0.1"`.
- Modify `src/main.rs`: register the `notifications` module.
- Modify `src/services.rs`: start and store `NotificationService`, then start the popup watcher.
- Modify `src/shell.rs`: store toast state, own per-monitor toast windows, and handle close/action messages.
- Modify `src/style.css`: add notification window and toast classes.
- Create `src/notifications/mod.rs`: exports notification submodules.
- Create `src/notifications/model.rs`: owned UI models and pure model tests.
- Create `src/notifications/service.rs`: async bridge from `NotificationService::popups` into `ShellMsg`.
- Create `src/notifications/window.rs`: GTK/layer-shell rendering and button event wiring.

## Notes Before Starting

The working tree currently has unstaged `Cargo.toml` and `Cargo.lock` edits. Do not overwrite them. Before editing dependencies, inspect:

```bash
git diff -- Cargo.toml Cargo.lock
```

The implementation should preserve existing user edits and only add the notification dependency and lockfile entries needed by this feature.

`NotificationService::new().await` returns `Result<Arc<NotificationService>, Error>`. Store the returned `Arc` directly instead of wrapping it in another `Arc`.

## Task 1: Add The Notification Model

**Files:**

- Modify: `Cargo.toml`
- Modify: `src/main.rs`
- Create: `src/notifications/mod.rs`
- Create: `src/notifications/model.rs`

- [ ] **Step 1: Add the dependency**

Add this dependency line to `Cargo.toml` with the other Wayle crates:

```toml
wayle-notification = "0.1"
```

Keep existing dependency edits intact.

- [ ] **Step 2: Register the module**

Add this line to `src/main.rs` with the other module declarations:

```rust
mod notifications;
```

- [ ] **Step 3: Create `src/notifications/mod.rs`**

```rust
pub(crate) mod model;
```

- [ ] **Step 4: Create the notification UI model**

Create `src/notifications/model.rs`:

```rust
use chrono::{DateTime, Utc};
use wayle_notification::core::notification::Notification;
use wayle_notification::types::Urgency;

const DEFAULT_ACTION_ID: &str = "default";
const FALLBACK_APP_NAME: &str = "Application";
const FALLBACK_ICON_NAME: &str = "dialog-information-symbolic";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotificationAction {
    pub(crate) id: String,
    pub(crate) label: String,
}

#[derive(Clone, Debug)]
pub(crate) struct NotificationToastFields {
    pub(crate) id: u32,
    pub(crate) app_name: Option<String>,
    pub(crate) app_icon: Option<String>,
    pub(crate) summary: String,
    pub(crate) body: Option<String>,
    pub(crate) actions: Vec<NotificationAction>,
    pub(crate) default_action: Option<NotificationAction>,
    pub(crate) urgency: Urgency,
    pub(crate) timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotificationToast {
    pub(crate) id: u32,
    pub(crate) app_name: String,
    pub(crate) app_icon: String,
    pub(crate) summary: String,
    pub(crate) body: Option<String>,
    pub(crate) actions: Vec<NotificationAction>,
    pub(crate) default_action: Option<NotificationAction>,
    pub(crate) urgency: Urgency,
    pub(crate) timestamp: DateTime<Utc>,
}

impl NotificationToast {
    pub(crate) fn from_notification(notification: &Notification) -> Self {
        Self::from_fields(NotificationToastFields {
            id: notification.id,
            app_name: notification.app_name.get(),
            app_icon: notification.app_icon.get(),
            summary: notification.summary.get(),
            body: notification.body.get(),
            actions: notification
                .actions
                .get()
                .into_iter()
                .map(|action| NotificationAction {
                    id: action.id,
                    label: action.label,
                })
                .collect(),
            default_action: notification.default_action.get().map(|action| NotificationAction {
                id: action.id,
                label: action.label,
            }),
            urgency: notification.urgency.get(),
            timestamp: notification.timestamp.get(),
        })
    }

    pub(crate) fn from_fields(fields: NotificationToastFields) -> Self {
        Self {
            id: fields.id,
            app_name: display_or_fallback(fields.app_name, FALLBACK_APP_NAME),
            app_icon: display_or_fallback(fields.app_icon, FALLBACK_ICON_NAME),
            summary: fields.summary,
            body: fields.body,
            actions: fields.actions,
            default_action: fields.default_action,
            urgency: fields.urgency,
            timestamp: fields.timestamp,
        }
    }

    pub(crate) fn visible_actions(&self) -> Vec<NotificationAction> {
        self.actions
            .iter()
            .filter(|action| action.id != DEFAULT_ACTION_ID)
            .cloned()
            .collect()
    }

    pub(crate) fn urgency_class(&self) -> &'static str {
        match self.urgency {
            Urgency::Low => "low",
            Urgency::Normal => "normal",
            Urgency::Critical => "critical",
        }
    }

    pub(crate) fn has_default_action(&self) -> bool {
        self.default_action.is_some()
    }
}

pub(crate) fn newest_first(mut toasts: Vec<NotificationToast>) -> Vec<NotificationToast> {
    toasts.sort_by(|left, right| {
        right
            .timestamp
            .cmp(&left.timestamp)
            .then_with(|| right.id.cmp(&left.id))
    });
    toasts
}

fn display_or_fallback(value: Option<String>, fallback: &str) -> String {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use super::*;

    #[test]
    fn from_fields_uses_fallbacks_for_blank_app_data() {
        let toast = NotificationToast::from_fields(fields(
            1,
            Some("  ".to_string()),
            Some("".to_string()),
            "Summary",
            "2026-06-04T12:00:00Z",
        ));

        assert_eq!(toast.app_name, FALLBACK_APP_NAME);
        assert_eq!(toast.app_icon, FALLBACK_ICON_NAME);
    }

    #[test]
    fn visible_actions_excludes_default_action() {
        let mut toast = NotificationToast::from_fields(fields(
            1,
            Some("Mail".to_string()),
            Some("mail-unread-symbolic".to_string()),
            "Message",
            "2026-06-04T12:00:00Z",
        ));
        toast.actions = vec![
            action("default", "Open"),
            action("reply", "Reply"),
            action("archive", "Archive"),
        ];

        assert_eq!(toast.visible_actions(), vec![action("reply", "Reply"), action("archive", "Archive")]);
    }

    #[test]
    fn urgency_class_matches_urgency() {
        let mut toast = NotificationToast::from_fields(fields(
            1,
            Some("Calendar".to_string()),
            Some("x-office-calendar-symbolic".to_string()),
            "Meeting",
            "2026-06-04T12:00:00Z",
        ));

        toast.urgency = Urgency::Critical;

        assert_eq!(toast.urgency_class(), "critical");
    }

    #[test]
    fn newest_first_orders_by_timestamp_then_id() {
        let oldest = NotificationToast::from_fields(fields(
            1,
            Some("App".to_string()),
            Some("dialog-information-symbolic".to_string()),
            "Old",
            "2026-06-04T12:00:00Z",
        ));
        let newer_low_id = NotificationToast::from_fields(fields(
            2,
            Some("App".to_string()),
            Some("dialog-information-symbolic".to_string()),
            "Newer low id",
            "2026-06-04T13:00:00Z",
        ));
        let newer_high_id = NotificationToast::from_fields(fields(
            3,
            Some("App".to_string()),
            Some("dialog-information-symbolic".to_string()),
            "Newer high id",
            "2026-06-04T13:00:00Z",
        ));

        let result = newest_first(vec![oldest, newer_low_id, newer_high_id]);

        assert_eq!(result.iter().map(|toast| toast.id).collect::<Vec<_>>(), vec![3, 2, 1]);
    }

    fn fields(
        id: u32,
        app_name: Option<String>,
        app_icon: Option<String>,
        summary: &str,
        timestamp: &str,
    ) -> NotificationToastFields {
        NotificationToastFields {
            id,
            app_name,
            app_icon,
            summary: summary.to_string(),
            body: Some("Body".to_string()),
            actions: Vec::new(),
            default_action: None,
            urgency: Urgency::Normal,
            timestamp: DateTime::parse_from_rfc3339(timestamp)
                .unwrap()
                .with_timezone(&Utc),
        }
    }

    fn action(id: &str, label: &str) -> NotificationAction {
        NotificationAction {
            id: id.to_string(),
            label: label.to_string(),
        }
    }
}
```

- [ ] **Step 5: Run model tests**

Run:

```bash
cargo test notifications::model
```

Expected: all four `notifications::model` tests pass.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs src/notifications/mod.rs src/notifications/model.rs
git commit -m "Add notification toast model"
```

## Task 2: Start The Notification Service And Watch Popups

**Files:**

- Modify: `src/notifications/mod.rs`
- Create: `src/notifications/service.rs`
- Modify: `src/services.rs`
- Modify: `src/shell.rs`

- [ ] **Step 1: Export the service module**

Update `src/notifications/mod.rs`:

```rust
pub(crate) mod model;
pub(crate) mod service;
```

- [ ] **Step 2: Create the popup watcher**

Create `src/notifications/service.rs`:

```rust
use std::sync::Arc;

use futures::StreamExt;
use relm4::Sender;
use wayle_notification::NotificationService;

use crate::notifications::model::{NotificationToast, newest_first};
use crate::shell::ShellMsg;

pub(crate) fn start(
    sender: Sender<ShellMsg>,
    service: Option<Arc<NotificationService>>,
) -> Option<relm4::JoinHandle<()>> {
    let Some(service) = service else {
        tracing::info!("Notification toasts disabled because notification service is unavailable");
        return None;
    };

    Some(relm4::spawn(async move {
        run(sender, service).await;
    }))
}

async fn run(sender: Sender<ShellMsg>, service: Arc<NotificationService>) {
    tracing::info!("Notification popup watcher started");

    send_popup_snapshot(&sender, service.as_ref());

    let mut popup_updates = service.popups.watch().fuse();

    while popup_updates.next().await.is_some() {
        send_popup_snapshot(&sender, service.as_ref());
    }

    tracing::info!("Notification popup watcher stopped");
    let _ = sender.send(ShellMsg::NotificationsChanged(Vec::new()));
}

fn send_popup_snapshot(sender: &Sender<ShellMsg>, service: &NotificationService) {
    let toasts = service
        .popups
        .get()
        .iter()
        .map(|notification| NotificationToast::from_notification(notification.as_ref()))
        .collect();

    let toasts = newest_first(toasts);

    if sender.send(ShellMsg::NotificationsChanged(toasts)).is_err() {
        tracing::error!("Failed to send notification popup snapshot");
    }
}
```

- [ ] **Step 3: Store and start `NotificationService`**

In `src/services.rs`, add this import:

```rust
use wayle_notification::NotificationService;
```

Add this field to `ShellServices`:

```rust
pub(crate) notification: Option<Arc<NotificationService>>,
```

Add this block in `init_shell_services()` after the Niri or power profile service block:

```rust
let notification = match NotificationService::new().await {
    Ok(service) => {
        tracing::info!("Notification service started");
        Some(service)
    }
    Err(error) => {
        tracing::error!("Failed to start notification service: {error}");
        None
    }
};
```

Include the field in the returned `ShellServices`:

```rust
ShellServices {
    audio,
    battery,
    brightness,
    niri,
    notification,
    power_profiles,
    systray,
}
```

In `start_all()`, start the watcher before the OSD watchers:

```rust
crate::notifications::service::start(input_sender.clone(), services.notification.clone());
```

- [ ] **Step 4: Add the shell message and stored list**

In `src/shell.rs`, add this field to `Shell`:

```rust
notifications: Vec<crate::notifications::model::NotificationToast>,
```

Add this message variant to `ShellMsg`:

```rust
NotificationsChanged(Vec<crate::notifications::model::NotificationToast>),
```

Initialize the field in `Shell::init`:

```rust
notifications: Vec::new(),
```

Handle the message in `Shell::update`:

```rust
ShellMsg::NotificationsChanged(notifications) => {
    self.notifications = notifications;
}
```

- [ ] **Step 5: Run the compile checkpoint**

Run:

```bash
cargo check
```

Expected: compile succeeds. Notification popups are not visible yet because no window renders them.

- [ ] **Step 6: Commit**

```bash
git add src/notifications/mod.rs src/notifications/service.rs src/services.rs src/shell.rs Cargo.toml Cargo.lock
git commit -m "Start notification popup watcher"
```

## Task 3: Build The Toast Window Renderer

**Files:**

- Modify: `src/notifications/mod.rs`
- Create: `src/notifications/window.rs`

- [ ] **Step 1: Export the window module**

Update `src/notifications/mod.rs`:

```rust
pub(crate) mod model;
pub(crate) mod service;
pub(crate) mod window;
```

- [ ] **Step 2: Create the toast window**

Create `src/notifications/window.rs`:

```rust
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;

use crate::notifications::model::NotificationToast;
use crate::shell::ShellMsg;

const TOP_MARGIN: i32 = 36;
const RIGHT_MARGIN: i32 = 12;
const STACK_SPACING: i32 = 8;

pub(crate) struct NotificationWindow {
    window: gtk::Window,
    stack: gtk::Box,
    sender: relm4::Sender<ShellMsg>,
}

impl NotificationWindow {
    pub(crate) fn new(monitor: &gtk::gdk::Monitor, sender: relm4::Sender<ShellMsg>) -> Self {
        let window = gtk::Window::new();
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_monitor(Some(monitor));
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, TOP_MARGIN);
        window.set_margin(Edge::Right, RIGHT_MARGIN);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_namespace(Some("wayward-notifications"));
        window.set_visible(false);
        window.add_css_class("notification-window");

        let stack = gtk::Box::new(gtk::Orientation::Vertical, STACK_SPACING);
        stack.add_css_class("notification-stack");
        stack.set_halign(gtk::Align::End);
        stack.set_valign(gtk::Align::Start);

        window.set_child(Some(&stack));

        Self {
            window,
            stack,
            sender,
        }
    }

    pub(crate) fn set_toasts(&self, toasts: &[NotificationToast]) {
        while let Some(child) = self.stack.first_child() {
            self.stack.remove(&child);
        }

        for toast in toasts {
            self.stack.append(&self.toast_widget(toast));
        }

        self.window.set_visible(!toasts.is_empty());
    }

    fn toast_widget(&self, toast: &NotificationToast) -> gtk::Widget {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.add_css_class("notification-toast");
        root.add_css_class(toast.urgency_class());

        root.append(&self.header(toast));
        root.append(&self.body(toast));

        let actions = toast.visible_actions();
        if !actions.is_empty() {
            root.append(&self.actions(toast.id, &actions));
        }

        root.upcast()
    }

    fn header(&self, toast: &NotificationToast) -> gtk::Widget {
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header.add_css_class("notification-header");

        let icon = gtk::Image::from_icon_name(&toast.app_icon);
        icon.add_css_class("notification-app-icon");
        header.append(&icon);

        let app_name = gtk::Label::new(Some(&toast.app_name));
        app_name.add_css_class("notification-app-name");
        app_name.set_hexpand(true);
        app_name.set_halign(gtk::Align::Start);
        app_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        header.append(&app_name);

        let close = gtk::Button::new();
        close.add_css_class("notification-close");
        close.add_css_class("flat");
        close.set_child(Some(&gtk::Image::from_icon_name("window-close-symbolic")));

        let sender = self.sender.clone();
        let id = toast.id;
        close.connect_clicked(move |_| {
            let _ = sender.send(ShellMsg::DismissNotificationPopup(id));
        });

        header.append(&close);
        header.upcast()
    }

    fn body(&self, toast: &NotificationToast) -> gtk::Widget {
        let body = gtk::Box::new(gtk::Orientation::Vertical, 4);
        body.add_css_class("notification-content");

        let summary = gtk::Label::new(Some(&toast.summary));
        summary.add_css_class("notification-summary");
        summary.set_halign(gtk::Align::Start);
        summary.set_wrap(true);
        summary.set_xalign(0.0);
        body.append(&summary);

        if let Some(text) = &toast.body {
            let label = gtk::Label::new(Some(text));
            label.add_css_class("notification-body");
            label.set_halign(gtk::Align::Start);
            label.set_wrap(true);
            label.set_xalign(0.0);
            body.append(&label);
        }

        if toast.has_default_action() {
            let sender = self.sender.clone();
            let id = toast.id;
            let gesture = gtk::GestureClick::new();

            gesture.connect_released(move |_, _, _, _| {
                let _ = sender.send(ShellMsg::InvokeNotificationDefaultAction(id));
            });

            body.add_controller(gesture);
            body.add_css_class("has-default-action");
        }

        body.upcast()
    }

    fn actions(&self, toast_id: u32, actions: &[crate::notifications::model::NotificationAction]) -> gtk::Widget {
        let row = gtk::FlowBox::new();
        row.add_css_class("notification-actions");
        row.set_selection_mode(gtk::SelectionMode::None);
        row.set_max_children_per_line(3);

        for action in actions {
            let button = gtk::Button::with_label(&action.label);
            button.add_css_class("notification-action");
            button.add_css_class("flat");

            let sender = self.sender.clone();
            let action_id = action.id.clone();

            button.connect_clicked(move |_| {
                let _ = sender.send(ShellMsg::InvokeNotificationAction {
                    id: toast_id,
                    action_id: action_id.clone(),
                });
            });

            row.insert(&button, -1);
        }

        row.upcast()
    }
}
```

- [ ] **Step 3: Run the compile checkpoint**

Run:

```bash
cargo check
```

Expected: compile succeeds. Rust may warn that `NotificationWindow` is not used yet.

- [ ] **Step 4: Commit**

```bash
git add src/notifications/mod.rs src/notifications/window.rs
git commit -m "Add notification toast window"
```

## Task 4: Wire Toast Windows Into Shell

**Files:**

- Modify: `src/shell.rs`

- [ ] **Step 1: Add the running window record**

In `src/shell.rs`, add this struct next to `RunningOsd`:

```rust
struct RunningNotificationWindow {
    connector: String,
    window: crate::notifications::window::NotificationWindow,
}
```

- [ ] **Step 2: Add shell state**

Add this field to `Shell`:

```rust
notification_windows: Vec<RunningNotificationWindow>,
```

Initialize it in `Shell::init`:

```rust
notification_windows: Vec::new(),
```

- [ ] **Step 3: Add shell messages for interactions**

Add these variants to `ShellMsg`:

```rust
DismissNotificationPopup(u32),
InvokeNotificationAction { id: u32, action_id: String },
InvokeNotificationDefaultAction(u32),
```

- [ ] **Step 4: Add reconciliation and rendering helpers**

Add these methods inside `impl Shell`:

```rust
fn reconcile_notification_windows(&mut self, sender: &ComponentSender<Self>) {
    let monitors = Self::available_monitors();

    self.notification_windows.retain(|running| {
        monitors
            .iter()
            .any(|monitor| monitor_connector(monitor).as_deref() == Some(running.connector.as_str()))
    });

    for monitor in monitors {
        let Some(connector) = monitor_connector(&monitor) else {
            continue;
        };

        if self
            .notification_windows
            .iter()
            .any(|running| running.connector == connector)
        {
            continue;
        }

        self.notification_windows.push(RunningNotificationWindow {
            connector,
            window: crate::notifications::window::NotificationWindow::new(
                &monitor,
                sender.input_sender().clone(),
            ),
        });
    }

    self.show_notifications();
}

fn show_notifications(&self) {
    let Some(focused_connector) = self.focused_monitor_connector.as_deref() else {
        for running in &self.notification_windows {
            running.window.set_toasts(&[]);
        }
        return;
    };

    for running in &self.notification_windows {
        if running.connector == focused_connector {
            running.window.set_toasts(&self.notifications);
        } else {
            running.window.set_toasts(&[]);
        }
    }
}

fn dismiss_notification_popup(&self, id: u32) {
    let Some(service) = self.services.notification.as_ref() else {
        tracing::info!("Cannot dismiss notification popup because notification service is unavailable");
        return;
    };

    service.dismiss_popup(id);
}

fn invoke_notification_action(&self, id: u32, action_id: String) {
    let Some(service) = self.services.notification.clone() else {
        tracing::info!("Cannot invoke notification action because notification service is unavailable");
        return;
    };

    relm4::spawn(async move {
        let notification = service
            .popups
            .get()
            .into_iter()
            .chain(service.notifications.get())
            .find(|notification| notification.id == id);

        if let Some(notification) = notification {
            if let Err(error) = notification.invoke(&action_id).await {
                tracing::error!(id, action_id, "Failed to invoke notification action: {error}");
            }
        } else {
            tracing::info!(id, action_id, "Notification action target disappeared");
        }

        service.dismiss_popup(id);
    });
}

fn invoke_notification_default_action(&self, id: u32) {
    let Some(service) = self.services.notification.clone() else {
        tracing::info!("Cannot invoke default notification action because notification service is unavailable");
        return;
    };

    relm4::spawn(async move {
        let notification = service
            .popups
            .get()
            .into_iter()
            .chain(service.notifications.get())
            .find(|notification| notification.id == id);

        if let Some(notification) = notification {
            if let Some(action) = notification.default_action.get() {
                let action_id = action.id;

                if let Err(error) = notification.invoke(&action_id).await {
                    tracing::error!(
                        id,
                        action_id = %action_id,
                        "Failed to invoke default notification action: {error}"
                    );
                }
            }
        } else {
            tracing::info!(id, "Default notification action target disappeared");
        }

        service.dismiss_popup(id);
    });
}
```

- [ ] **Step 5: Call reconciliation during init and monitor changes**

In `Shell::init`, call this after `model.reconcile_osd_windows();`:

```rust
model.reconcile_notification_windows(&sender);
```

In the `ShellMsg::ReconcileMonitors` branch, call this after `self.reconcile_osd_windows();`:

```rust
self.reconcile_notification_windows(&_sender);
```

- [ ] **Step 6: Re-render when focus or notifications change**

In the `ShellMsg::ItemStateChanged(state)` branch, call this after storing and forwarding the state:

```rust
self.show_notifications();
```

Replace the temporary `NotificationsChanged` handler from Task 2 with:

```rust
ShellMsg::NotificationsChanged(notifications) => {
    self.notifications = notifications;
    self.show_notifications();
}
```

- [ ] **Step 7: Handle close and action messages**

Add these branches to `Shell::update`:

```rust
ShellMsg::DismissNotificationPopup(id) => {
    self.dismiss_notification_popup(id);
}
ShellMsg::InvokeNotificationAction { id, action_id } => {
    self.invoke_notification_action(id, action_id);
}
ShellMsg::InvokeNotificationDefaultAction(id) => {
    self.invoke_notification_default_action(id);
}
```

- [ ] **Step 8: Run the compile checkpoint**

Run:

```bash
cargo check
```

Expected: compile succeeds. Toast windows now exist, but styling is still bare.

- [ ] **Step 9: Commit**

```bash
git add src/shell.rs
git commit -m "Show notification toasts on focused monitor"
```

## Task 5: Add Notification Styling

**Files:**

- Modify: `src/style.css`

- [ ] **Step 1: Add CSS below the OSD section**

Append this CSS to `src/style.css`:

```css
/* Notifications */
.notification-window {
    background: transparent;
}

.notification-stack {
    background: transparent;
    min-width: 18em;
}

.notification-toast {
    background: rgba(32, 33, 36, 1.00);
    border: 1px solid rgba(241, 243, 244, 0.16);
    border-radius: 0.6em;
    color: #f1f3f4;
    min-width: 18em;
    padding: 0;
}

.notification-toast.critical {
    border-color: #f28b82;
}

.notification-header {
    border-bottom: 1px solid rgba(241, 243, 244, 0.10);
    padding: 0.55em 0.65em;
}

.notification-app-icon {
    -gtk-icon-size: 1.1em;
    min-width: 1.1em;
}

.notification-app-name {
    font-weight: 600;
}

.notification-close {
    background: transparent;
    background-image: none;
    border: none;
    box-shadow: none;
    min-height: 0;
    min-width: 0;
    padding: 0.15em;
}

.notification-content {
    padding: 0.65em;
}

.notification-summary {
    font-weight: 600;
}

.notification-body {
    opacity: 0.82;
}

.notification-actions {
    border-top: 1px solid rgba(241, 243, 244, 0.10);
    padding: 0.5em 0.65em 0.65em;
}

.notification-action {
    background: transparent;
    background-image: none;
    border: 1px solid rgba(241, 243, 244, 0.22);
    border-radius: 0.35em;
    box-shadow: none;
    color: inherit;
    min-height: 0;
    padding: 0.3em 0.55em;
}

.notification-action:hover {
    border-color: #8ab4f8;
}
```

- [ ] **Step 2: Run formatting and compile check**

Run:

```bash
cargo fmt
cargo check
```

Expected: formatting succeeds and compile succeeds.

- [ ] **Step 3: Commit**

```bash
git add src/style.css
git commit -m "Style notification toasts"
```

## Task 6: Runtime Verification

**Files:**

- No source files unless verification reveals a compile or runtime issue.

- [ ] **Step 1: Confirm notification test tool exists**

Run:

```bash
command -v notify-send
```

Expected: prints a path such as `/usr/bin/notify-send`.

If it prints nothing, install or choose another local notification sender before continuing.

- [ ] **Step 2: Start Wayward**

Run the normal local app command:

```bash
cargo run
```

Expected: Wayward starts without a notification service error. If another notification daemon owns `org.freedesktop.Notifications`, stop that daemon and restart Wayward.

- [ ] **Step 3: Send a plain notification**

In another terminal, run:

```bash
notify-send "Wayward test" "Plain notification body"
```

Expected: an App Header toast appears on the focused monitor, top-right below the bar.

- [ ] **Step 4: Verify newest-first stacking**

Run:

```bash
notify-send "Wayward one" "Older toast"
notify-send "Wayward two" "Newest toast"
```

Expected: `Wayward two` appears above `Wayward one`.

- [ ] **Step 5: Verify close button behavior**

Click the toast close button.

Expected: the toast disappears. Wayward keeps running and logs no panic.

- [ ] **Step 6: Verify action buttons**

Run:

```bash
notify-send --action=reply=Reply --action=archive=Archive "Wayward actions" "Choose an action"
```

Expected: the toast shows `Reply` and `Archive` buttons. Clicking either button hides the popup after invoking the action.

- [ ] **Step 7: Verify default body click**

Run:

```bash
notify-send --action=default=Open "Wayward default" "Click the body"
```

Expected: clicking the toast body invokes the default action and hides the popup.

- [ ] **Step 8: Verify focused monitor routing**

Move focus to a workspace on another monitor, then run:

```bash
notify-send "Wayward focus" "This should follow focus"
```

Expected: the toast appears on the newly focused monitor.

- [ ] **Step 9: Final verification**

Stop `cargo run`, then run:

```bash
cargo fmt
cargo test notifications::model
cargo check
```

Expected: all commands succeed.

- [ ] **Step 10: Commit runtime fixes if any were needed**

If runtime verification required source changes, commit them:

```bash
git add src Cargo.toml Cargo.lock
git commit -m "Fix notification toast runtime behavior"
```

If no fixes were needed, do not create an empty commit.
