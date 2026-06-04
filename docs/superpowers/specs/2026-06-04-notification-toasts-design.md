# Notification Toasts Design

Date: 2026-06-04

## Goal

Add first-pass notification toast support to Wayward using `wayle-notification`.

The feature displays transient notification popups on the focused monitor. It does not add the later notification history dropdown yet, but it keeps history support in mind by using the crate's `popups` stream for visible toasts and leaving `notifications` for future history UI.

## Scope

Included:

- Start `wayle_notification::NotificationService` with the other shell services.
- Watch `NotificationService::popups` and render visible toasts.
- Show toasts on the focused monitor only.
- Anchor toasts top-right below the bar.
- Stack newest notifications first, pushing older toasts downward.
- Render the App Header toast layout.
- Render close, action buttons, and body default-action clicks.
- Hide popups without removing notification history.

Excluded for now:

- Notification history dropdown.
- DND controls in the Wayward UI.
- Per-monitor or user-configurable toast placement.
- Rich image data rendering.
- Notification body markup rendering.
- Grouping or collapsing notifications by app.

## Architecture

Notifications are a shell-level overlay system, not a bar widget. This matches the existing OSD direction: the shell owns floating layer-shell windows per monitor and decides which monitor should show transient UI.

`src/services.rs` will store notification support on `ShellServices`:

```rust
pub(crate) notification: Option<Arc<NotificationService>>,
```

`init_shell_services()` starts `NotificationService`. The first pass can use `NotificationService::new().await`; `with_daemon()` can be added later if Wayward needs CLI control over notification state.

`Shell` will keep one notification toast window per monitor, similar to `RunningOsd`. The shell already tracks `focused_monitor_connector` from workspace state. That focused connector determines which toast window gets the current toast stack.

The notification code should live in a focused module:

- `src/notifications/mod.rs` exports the module API and message types.
- `src/notifications/service.rs` watches `NotificationService::popups`.
- `src/notifications/model.rs` converts service notifications into UI summaries.
- `src/notifications/window.rs` owns GTK and layer-shell rendering.

This keeps `shell.rs` responsible for orchestration while notification-specific modeling and rendering stay local to the notification module.

## Data Flow

At startup:

1. `init_shell_services()` creates `NotificationService`.
2. `services::start_all()` starts a notification watcher if the service is available.
3. The watcher sends `ShellMsg::NotificationsChanged(Vec<NotificationToast>)` when `service.popups.watch()` changes.

`NotificationToast` is an owned UI model. It should copy display fields out of the service notification:

- `id`
- `app_name`
- `app_icon`
- `summary`
- `body`
- `actions`
- `default_action`
- `urgency`
- `timestamp`

The watcher normalizes the list so the newest toast appears first. `Shell` stores the latest list and sends it to the currently focused monitor's toast window. Other monitor windows receive an empty list or are hidden.

The toast window reconciles rows by notification ID. This lets content update without rebuilding unrelated rows.

Interactions flow back through `ShellMsg`:

- `DismissNotificationPopup(id)`
- `InvokeNotificationAction { id, action_id }`
- `InvokeNotificationDefaultAction(id)`

For action invocation, the shell looks up the current service notification by ID and calls `notification.invoke(action_id).await`. After the invocation attempt, the shell hides the popup immediately with `service.dismiss_popup(id)`.

For close, the shell calls `service.dismiss_popup(id)` only. This hides the toast while keeping the notification available for the later history dropdown.

## UI Behavior

Toasts appear on the focused monitor only, anchored top-right below the bar. New notifications appear at the top of the stack and push older notifications downward.

Each toast uses the App Header layout:

- Header row: app icon or fallback icon, app name or fallback label, close button.
- Body area: summary and optional body text.
- Action row: rendered only when there are non-default actions.
- Body click: invokes the default action when present.
- Close button: hides the popup but keeps notification history.
- Action button: invokes that action and immediately hides the popup.

For icon handling, the first pass uses `app_icon` as a GTK icon name when present. If it is absent or cannot resolve, the UI falls back to a generic notification icon.

For body handling, the first pass renders summary and body as plain wrapped labels. Notification bodies may contain markup, but rendering plain text avoids letting arbitrary notification text affect GTK label markup before sanitization is designed.

## Error Handling

If `NotificationService` fails to start, notification toasts are disabled and the shell logs the startup error. The rest of Wayward continues running.

If the popup stream ends, the watcher logs that notification updates stopped and hides toast windows by sending an empty list or a stopped message.

If no focused monitor is known yet, `Shell` stores the latest toast list but does not show it. When workspace state later identifies a focused monitor, the focused monitor's window renders the current stack.

If action invocation fails, the shell logs the error and still hides the popup. The user clicked an action, so keeping a failed action toast on screen would be noisy for the first pass.

If a notification has many actions, the first pass renders all non-default actions in a wrapping row. A maximum or overflow menu can be added later if real notifications need it.

## Styling

Default styling should use the existing color and density direction from `style.css`.

Suggested CSS classes:

- `notification-window`
- `notification-stack`
- `notification-toast`
- `notification-header`
- `notification-app-icon`
- `notification-app-name`
- `notification-close`
- `notification-summary`
- `notification-body`
- `notification-actions`
- `notification-action`
- urgency classes such as `low`, `normal`, and `critical`

The window itself should stay transparent. Toast cards should carry their own background, border, padding, and spacing.

## Testing And Verification

Unit tests should focus on model and interaction policy:

- Convert a notification snapshot into `NotificationToast`.
- Sort toasts newest first.
- Filter default actions out of the rendered action row.
- Choose fallback app label and fallback icon when fields are missing.

Manual verification should cover:

- Send a test notification and see it appear on the focused monitor.
- Send multiple notifications and confirm newest appears at the top.
- Click close and confirm the popup disappears without removing future history.
- Click an action button and confirm the action fires, then the popup hides.
- Click the body when a default action exists and confirm it invokes the default action.
- Move focus to another monitor and confirm new toasts target that monitor.

Useful commands for manual notification testing will be added to the implementation plan after confirming local tooling.

## Upstream API Notes

`wayle-notification` exposes:

- `NotificationService::popups` for currently visible popups.
- `NotificationService::notifications` for all received notifications.
- `NotificationService::dismiss_popup(id)` to hide a popup without removing history.
- `Notification::dismiss()` to remove a notification from history.
- `Notification::invoke(action_key).await` to emit an action signal.

This design intentionally uses `popups` and `dismiss_popup(id)` for toast UI so the future notification history dropdown can use `notifications`.
