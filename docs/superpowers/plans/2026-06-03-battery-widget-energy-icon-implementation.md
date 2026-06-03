# Battery Widget Energy Icon Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show the battery widget as an icon, percentage, and watts while keeping the existing power profile dropdown.

**Architecture:** `BatterySnapshot` remains the state boundary between async battery data and GTK rendering. Formatting and icon-selection helpers stay pure and testable, while `BatteryRuntime` owns the GTK widgets that display icon, percentage, and energy rate.

**Tech Stack:** Rust 2024, Relm4, GTK4, futures streams, `wayle-battery`, `wayle-power-profiles`.

---

## File Structure

- Modify: `src/bar/state.rs` - add `energy_rate` to `BatterySnapshot`.
- Modify: `src/bar/widgets/battery.rs` - add helper tests, snapshot field population, watcher stream, and GTK rendering changes.
- Modify: `src/style.css` - add compact classes for icon, percentage, and energy rate inside the battery bar item.

## Task 1: Add Pure Battery Display Helpers

**Files:**
- Modify: `src/bar/widgets/battery.rs`

- [ ] **Step 1: Add helper functions below `battery_text`**

Find this existing function:

```rust
fn battery_text(percentage: f64, state: DeviceState) -> String {
    format!("{percentage:.0}% {state}")
}
```

Add these helper functions immediately below it:

```rust
fn battery_percentage_text(percentage: f64) -> String {
    format!("{percentage:.0}%")
}

fn battery_energy_rate_text(energy_rate: f64) -> String {
    format!("{energy_rate:.1}W")
}

fn battery_icon_name(percentage: f64, state: DeviceState) -> &'static str {
    let level = ((percentage / 10.0).round() as i32 * 10).clamp(0, 100);

    match state {
        DeviceState::FullyCharged => "battery-level-100-charged-symbolic",
        DeviceState::Charging => charging_battery_icon_name(level),
        _ => discharging_battery_icon_name(level),
    }
}

fn charging_battery_icon_name(level: i32) -> &'static str {
    match level {
        100 => "battery-level-100-charging-symbolic",
        90 => "battery-level-90-charging-symbolic",
        80 => "battery-level-80-charging-symbolic",
        70 => "battery-level-70-charging-symbolic",
        60 => "battery-level-60-charging-symbolic",
        50 => "battery-level-50-charging-symbolic",
        40 => "battery-level-40-charging-symbolic",
        30 => "battery-level-30-charging-symbolic",
        20 => "battery-level-20-charging-symbolic",
        10 => "battery-level-10-charging-symbolic",
        _ => "battery-level-0-charging-symbolic",
    }
}

fn discharging_battery_icon_name(level: i32) -> &'static str {
    match level {
        100 => "battery-level-100-symbolic",
        90 => "battery-level-90-symbolic",
        80 => "battery-level-80-symbolic",
        70 => "battery-level-70-symbolic",
        60 => "battery-level-60-symbolic",
        50 => "battery-level-50-symbolic",
        40 => "battery-level-40-symbolic",
        30 => "battery-level-30-symbolic",
        20 => "battery-level-20-symbolic",
        10 => "battery-level-10-symbolic",
        _ => "battery-level-0-symbolic",
    }
}
```

Rust concept: `&'static str` means the returned string slice lives for the whole program. String literals have this lifetime, so they are perfect for icon names.

- [ ] **Step 2: Add focused helper tests at the bottom of the file**

Append this test module after `battery_message`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_percentage_text_rounds_to_whole_percent() {
        assert_eq!(battery_percentage_text(87.4), "87%");
        assert_eq!(battery_percentage_text(87.5), "88%");
    }

    #[test]
    fn battery_energy_rate_text_formats_watts_directly() {
        assert_eq!(battery_energy_rate_text(6.24), "6.2W");
        assert_eq!(battery_energy_rate_text(0.04), "0.0W");
    }

    #[test]
    fn battery_icon_name_uses_charged_icon_for_fully_charged() {
        assert_eq!(
            battery_icon_name(100.0, DeviceState::FullyCharged),
            "battery-level-100-charged-symbolic"
        );
    }

    #[test]
    fn battery_icon_name_uses_charging_icons_for_charging_state() {
        assert_eq!(
            battery_icon_name(84.0, DeviceState::Charging),
            "battery-level-80-charging-symbolic"
        );
    }

    #[test]
    fn battery_icon_name_uses_level_icons_for_other_states() {
        assert_eq!(
            battery_icon_name(26.0, DeviceState::Discharging),
            "battery-level-30-symbolic"
        );
    }
}
```

Rust concept: unit tests inside the same file can use private functions through `use super::*;`. That is why these helpers do not need to be `pub`.

- [ ] **Step 3: Run the focused test target**

Run:

```bash
cargo test battery_
```

Expected: the five new helper tests pass.

- [ ] **Step 4: Commit the helper tests and functions**

Run:

```bash
git add src/bar/widgets/battery.rs
git commit -m "test: add battery display helpers"
```

## Task 2: Carry Energy Rate Through BatterySnapshot

**Files:**
- Modify: `src/bar/state.rs`
- Modify: `src/bar/widgets/battery.rs`

- [ ] **Step 1: Add the snapshot field**

In `src/bar/state.rs`, change `BatterySnapshot` from:

```rust
pub(crate) struct BatterySnapshot {
    pub(crate) percentage: f64,
    pub(crate) state: DeviceState,
    pub(crate) active_profile: Option<PowerProfile>,
    pub(crate) available_profiles: Vec<PowerProfile>,
}
```

to:

```rust
pub(crate) struct BatterySnapshot {
    pub(crate) percentage: f64,
    pub(crate) state: DeviceState,
    pub(crate) energy_rate: f64,
    pub(crate) active_profile: Option<PowerProfile>,
    pub(crate) available_profiles: Vec<PowerProfile>,
}
```

Rust concept: Rust struct literals must initialize every field. This change intentionally creates a compile error until `send_battery_snapshot` fills the new value.

- [ ] **Step 2: Populate the new field**

In `send_battery_snapshot`, change:

```rust
let percentage = service.device.percentage.get();
let state = service.device.state.get();
```

to:

```rust
let percentage = service.device.percentage.get();
let state = service.device.state.get();
let energy_rate = service.device.energy_rate.get();
```

Then change the snapshot literal from:

```rust
let snapshot = BatterySnapshot {
    percentage,
    state,
    active_profile,
    available_profiles,
};
```

to:

```rust
let snapshot = BatterySnapshot {
    percentage,
    state,
    energy_rate,
    active_profile,
    available_profiles,
};
```

- [ ] **Step 3: Run the compiler**

Run:

```bash
cargo check
```

Expected: this should pass, but the bar will not refresh on energy rate changes yet.

- [ ] **Step 4: Commit snapshot field plumbing**

Run:

```bash
git add src/bar/state.rs src/bar/widgets/battery.rs
git commit -m "feat: carry battery energy rate in state"
```

## Task 3: Watch Energy Rate Changes

**Files:**
- Modify: `src/bar/widgets/battery.rs`

- [ ] **Step 1: Add an energy rate stream**

In `run_battery_watcher`, find:

```rust
let mut percentage_updates = service.device.percentage.watch().fuse();
let mut state_updates = service.device.state.watch().fuse();
```

Change it to:

```rust
let mut percentage_updates = service.device.percentage.watch().fuse();
let mut state_updates = service.device.state.watch().fuse();
let mut energy_rate_updates = service.device.energy_rate.watch().fuse();
```

- [ ] **Step 2: Add a `select!` branch**

Inside the `select!` block, after the `state_updates` branch, add:

```rust
update = energy_rate_updates.next() => {
    if update.is_none() {
        break;
    }

    send_battery_snapshot(&sender, &service, power_profiles.as_deref());
}
```

Rust concept: each `.watch()` stream yields when that property changes. Without this branch, `energy_rate.get()` would be correct only when some other watched property happened to trigger a new snapshot.

- [ ] **Step 3: Run the compiler**

Run:

```bash
cargo check
```

Expected: pass.

- [ ] **Step 4: Commit the watcher update**

Run:

```bash
git add src/bar/widgets/battery.rs
git commit -m "feat: refresh battery widget on energy changes"
```

## Task 4: Render Icon, Percentage, And Watts

**Files:**
- Modify: `src/bar/widgets/battery.rs`

- [ ] **Step 1: Expand the GTK prelude imports**

Change this import:

```rust
use relm4::gtk::prelude::{ToggleButtonExt, WidgetExt};
```

to:

```rust
use relm4::gtk::prelude::{BoxExt, ToggleButtonExt, WidgetExt};
```

GTK concept: `BoxExt` provides methods such as `.append(...)` on `gtk::Box`.

- [ ] **Step 2: Change the runtime fields**

Change:

```rust
struct BatteryRuntime {
    root: gtk::MenuButton,
    label: gtk::Label,
    dropdown: crate::bar::dropdown::Dropdown,
    profile_buttons: Vec<(PowerProfile, gtk::ToggleButton)>,
}
```

to:

```rust
struct BatteryRuntime {
    root: gtk::MenuButton,
    icon: gtk::Image,
    percentage_label: gtk::Label,
    energy_rate_label: gtk::Label,
    dropdown: crate::bar::dropdown::Dropdown,
    profile_buttons: Vec<(PowerProfile, gtk::ToggleButton)>,
}
```

- [ ] **Step 3: Update unavailable rendering**

In `BatteryRuntime::update`, replace:

```rust
self.label.set_text(&initial_text());
```

with:

```rust
self.icon.set_icon_name(Some("battery-missing-symbolic"));
self.percentage_label.set_text(&initial_text());
self.energy_rate_label.set_text("");
```

- [ ] **Step 4: Update ready rendering**

Replace:

```rust
let text = battery_text(snapshot.percentage, snapshot.state);
self.label.set_text(&text);
```

with:

```rust
self.icon
    .set_icon_name(Some(battery_icon_name(snapshot.percentage, snapshot.state)));
self.percentage_label
    .set_text(&battery_percentage_text(snapshot.percentage));
self.energy_rate_label
    .set_text(&battery_energy_rate_text(snapshot.energy_rate));
```

- [ ] **Step 5: Build the visible button content**

In `BatteryWidget::build`, replace:

```rust
let label = gtk::Label::new(Some(&initial_text()));
label.add_css_class("battery-label");
```

with:

```rust
let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
content.add_css_class("battery-content");

let icon = gtk::Image::from_icon_name("battery-missing-symbolic");
icon.add_css_class("battery-icon");
icon.set_pixel_size(16);
content.append(&icon);

let percentage_label = gtk::Label::new(Some(&initial_text()));
percentage_label.add_css_class("battery-percentage");
content.append(&percentage_label);

let energy_rate_label = gtk::Label::new(None);
energy_rate_label.add_css_class("battery-energy-rate");
content.append(&energy_rate_label);
```

Then change the dropdown menu button call from:

```rust
&label,
```

to:

```rust
&content,
```

Finally, change the runtime construction from:

```rust
Box::new(BatteryRuntime {
    root,
    label,
    dropdown,
    profile_buttons,
})
```

to:

```rust
Box::new(BatteryRuntime {
    root,
    icon,
    percentage_label,
    energy_rate_label,
    dropdown,
    profile_buttons,
})
```

- [ ] **Step 6: Remove the old combined text helper**

Delete this function:

```rust
fn battery_text(percentage: f64, state: DeviceState) -> String {
    format!("{percentage:.0}% {state}")
}
```

Rust concept: after this render split, there is no single combined battery label. Removing the function keeps warnings useful.

- [ ] **Step 7: Run the compiler**

Run:

```bash
cargo check
```

Expected: pass. If GTK says `set_icon_name` expects `Option<&str>`, confirm the calls look like `Some("...")` and `Some(battery_icon_name(...))`.

- [ ] **Step 8: Commit rendering change**

Run:

```bash
git add src/bar/widgets/battery.rs
git commit -m "feat: render battery icon percentage and watts"
```

## Task 5: Add Battery Content Styling

**Files:**
- Modify: `src/style.css`

- [ ] **Step 1: Add CSS after `.battery-dropdown-title`**

Find:

```css
.battery-dropdown-title {
    margin-bottom: 4px;
}
```

Add:

```css
.battery-content {
    min-height: 0;
}

.battery-icon {
    min-width: 16px;
}

.battery-percentage,
.battery-energy-rate {
    min-width: 0;
}
```

CSS concept: the icon gets a stable minimum width so changing from one battery icon to another does not shift the text.

- [ ] **Step 2: Run formatting and compiler checks**

Run:

```bash
cargo fmt
cargo check
```

Expected: both pass.

- [ ] **Step 3: Manually verify runtime behavior**

Run the app the same way you have been running it locally.

Expected in the bar:

```text
[battery icon] 87% 6.2W
```

Expected in the dropdown:

- Power profile buttons still appear.
- Active profile is still checked.
- Clicking a profile still changes the active profile.

- [ ] **Step 4: Commit styling**

Run:

```bash
git add src/style.css
git commit -m "style: align battery energy display"
```

## Task 6: Final Verification

**Files:**
- No source edits expected.

- [ ] **Step 1: Run final checks**

Run:

```bash
cargo fmt --check
cargo check
cargo test battery_
```

Expected: all commands pass.

- [ ] **Step 2: Inspect the commit history**

Run:

```bash
git log --oneline -5
```

Expected: the recent commits show helper tests, state plumbing, watcher refresh, rendering, and styling as separate commits.

- [ ] **Step 3: Record runtime observations if needed**

If the icon name does not resolve in the running bar, test a local theme fallback by changing the helper to return `battery-full-symbolic` or `battery-symbolic` for one case and rerun the app.

Expected: the app still compiles, and the icon issue is isolated to icon theme availability rather than GTK rendering.
