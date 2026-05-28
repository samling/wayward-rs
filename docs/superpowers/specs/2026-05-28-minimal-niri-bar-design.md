# Minimal Niri Workspace Bar Design

## Goal

Build the smallest useful Rust/Relm4 shell surface for Wayward: a read-only Wayland bar that shows all Niri workspaces.

The first milestone is intentionally narrow. It should prove that the app can connect to Niri through `wayle-niri`, receive reactive workspace updates, and render those workspaces in a GTK UI.

## Scope

Included:

- A Rust binary crate named `wayward`.
- A Relm4 GTK application with one bar window.
- Integration with `wayle-niri`.
- Read-only rendering of all Niri workspaces.
- Minimal visual state for normal, active, focused, and urgent workspaces.

Excluded for now:

- Clicking a workspace to focus it.
- Configuration files.
- Per-output bars.
- Other shell services such as audio, network, battery, notifications, tray, or wallpaper.
- Custom theming beyond a small internal CSS baseline.

## Architecture

The app will use three small modules:

- `main.rs` starts the Relm4 application and wires the root component.
- `niri.rs` owns the `wayle_niri::NiriService` connection and converts external workspace data into local view models.
- `bar.rs` owns the Relm4 component and renders workspace summaries.

The local view model keeps the UI independent from the exact `wayle-niri` workspace type:

```rust
struct WorkspaceSummary {
    id: u64,
    idx: u8,
    name: Option<String>,
    output: Option<String>,
    is_active: bool,
    is_focused: bool,
    is_urgent: bool,
}
```

## Data Flow

On startup, the app connects to Niri with `NiriService::new().await`.

The service exposes `workspaces` as a reactive property. The adapter takes an initial snapshot with `.get()`, sends it to the bar component, then watches for updates with `.watch()`.

Each update is converted into a `Vec<WorkspaceSummary>`. The bar receives that vector as a message, stores it in component state, and re-renders the workspace row.

Workspaces are displayed in the order provided by `wayle-niri`. Each workspace label shows its `name` when present, otherwise its `idx`.

## UI

The first UI is deliberately plain:

- A horizontal workspace row.
- One label or disabled button per workspace.
- CSS classes for `workspace`, `active`, `focused`, and `urgent`.
- Optional output text can be kept out of the first visual pass unless debugging makes it useful.

The app should use GTK4 layer shell support so it behaves like a bar rather than a normal application window.

## Error Handling

If Niri is unavailable or `$NIRI_SOCKET` is missing, the app should show a small fallback row saying that Niri is unavailable and log the error.

If the workspace stream ends, the bar should keep the last known snapshot and log that updates stopped.

## Testing

The first tests should avoid requiring a live Niri session:

- Unit test conversion from service workspace snapshots into `WorkspaceSummary`.
- Unit test workspace labels prefer `name` over `idx`.
- Manual verification inside a Niri session after the app builds.

## Learning Approach

The implementation should be written in small guided steps. Codex can explain each piece, provide focused examples, and review code, but should avoid generating the whole app at once.
