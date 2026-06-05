# Battery Dropdown Meter Design

Date: 2026-06-04

## Goal

Improve the battery dropdown so it is more useful than a power-profile selector alone.

The dropdown should lead with a meter-first battery summary and keep the existing power profile controls underneath it.

## Scope

Included:

- Add a visual charge-level indicator to the battery dropdown.
- Show charge percentage in the dropdown.
- Show energy rate in the dropdown.
- Show battery health in the dropdown.
- Keep the existing power profile selector in the dropdown.
- Keep the compact bar item unchanged.

Excluded for now:

- Time remaining estimates.
- Charge threshold controls.
- Historical battery charts.
- Per-device battery selection.
- Battery dropdown configuration.

## Architecture

The existing battery widget has the right ownership boundaries.

`BatterySnapshot` remains the async-to-UI boundary for battery data. The watcher in `src/bar/widgets/battery/service.rs` reads reactive `wayle-battery` properties and sends a snapshot through `BarItemState::Battery`.

`src/bar/widgets/battery/mod.rs` owns runtime updates. It updates the bar item and dropdown widgets from the same snapshot.

`src/bar/widgets/battery/dropdown.rs` owns dropdown construction. It should return a small handle struct containing:

- The dropdown root widget.
- Charge meter widget or fill widget.
- Charge percentage label.
- Energy rate label.
- Battery health label.
- Existing power profile buttons.

This keeps widget construction local to `dropdown.rs` while avoiding GTK child traversal for every dynamic dropdown value.

## Data Flow

Extend `BatterySnapshot` with:

```rust
pub(crate) capacity: f64,
```

`wayle-battery` exposes `device.capacity` as the battery capacity percentage between 0 and 100. This is the health value displayed in the dropdown.

The watcher should:

- Read `service.device.capacity.get()` when building a snapshot.
- Watch `service.device.capacity.watch().fuse()` so health updates reach the UI.
- Continue watching percentage, state, energy rate, and power profile values as it does now.

The runtime update path should:

- Continue updating the bar icon, percentage, and energy rate.
- Update the dropdown percentage label from `snapshot.percentage`.
- Update the dropdown energy-rate label from `snapshot.energy_rate`.
- Update the dropdown health label from `snapshot.capacity`.
- Update the dropdown visual meter from `snapshot.percentage`.
- Continue updating power-profile button availability and active state.

## UI Behavior

The dropdown uses the meter-first direction:

1. Title: `Battery`.
2. Summary row with a large battery indicator and large percentage.
3. Detail row with `Energy rate` and `Health`.
4. Existing `Power profile` title and segmented buttons.

The visual indicator should communicate the charge level at a glance. A GTK `LevelBar` is a good first choice because it already represents a bounded level and can be styled in CSS. If GTK styling becomes awkward, a box with a fill child can replace it without changing the runtime data flow.

For unavailable battery state, the dropdown should show fallback values:

- Percentage: the existing initial text policy.
- Energy rate: empty or unavailable text.
- Health: unavailable text.
- Meter: empty or zero.
- Power-profile buttons: disabled.

## Formatting

Existing helpers should be reused where possible:

- `battery_percentage_text(percentage)`
- `battery_energy_rate_text(energy_rate)`

Add a health formatter near the other battery formatting helpers:

```rust
pub(super) fn battery_health_text(capacity: f64) -> String {
    format!("{capacity:.0}%")
}
```

Do not add new state text in the first pass. The requested dropdown values are charge level, percentage, energy rate, health, and power profile controls.

## Styling

Presentation belongs in `src/style.css`.

Suggested CSS classes:

- `battery-summary`
- `battery-meter-row`
- `battery-meter`
- `battery-dropdown-percentage`
- `battery-details`
- `battery-detail`
- `battery-detail-label`
- `battery-detail-value`

The dropdown should continue using the existing `.dropdown-content` and `.battery-dropdown-content` hooks. CSS should own spacing, color, and sizing. Rust should set text and meter values only.

## Testing And Verification

Useful unit tests:

- `battery_health_text` rounds to a whole percentage.
- `battery_percentage_text` and `battery_energy_rate_text` tests continue passing.

Useful verification commands:

```sh
cargo fmt
cargo test battery
cargo check
```

Manual verification:

- Open the battery dropdown and confirm the meter, percentage, energy rate, health, and power profile controls appear.
- Confirm the bar item still shows icon, percentage, and energy rate.
- Confirm health changes would be watched by the service path.
- Confirm unavailable battery state disables profile controls and does not leave stale dropdown values.
