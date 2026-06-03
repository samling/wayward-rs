# Battery Widget Energy And Icon Design

Date: 2026-06-03

## Goal

Update the battery bar widget so the visible bar item shows battery state as an icon, percentage, and energy rate:

```text
[battery icon] 87% 6.2W
```

The dropdown remains focused on power profiles for this pass. Time remaining, capacity, and other detailed fields can be added later without changing the shape of this feature.

## User Workflow

The user opens the app and reads the battery item directly from the bar. The item should no longer show textual states such as `Fully Charged`; the state is represented by a theme icon. The percentage remains visible, and the energy rate is shown next to it in watts.

## Architecture

`BatterySnapshot` remains the boundary between the async battery watcher and GTK rendering. It gains one field:

```rust
pub(crate) energy_rate: f64,
```

The watcher reads `service.device.energy_rate.get()` when building the snapshot. It also listens to `service.device.energy_rate.watch()` so a change in power draw refreshes the bar even if percentage and UPower state do not change.

The GTK runtime changes from one label child to a horizontal box child inside the existing `gtk::MenuButton`. The runtime stores:

```rust
icon: gtk::Image,
percentage_label: gtk::Label,
energy_rate_label: gtk::Label,
```

This keeps icon, percentage, and energy rate independently updateable.

## Icon Naming

Use GTK icon theme names with `gtk::Image::from_icon_name(...)` and `image.set_icon_name(Some(name))`. The initial implementation uses common symbolic battery names such as:

- `battery-level-100-charged-symbolic`
- `battery-level-90-symbolic`
- `battery-level-80-symbolic`
- `battery-level-70-symbolic`
- `battery-level-60-symbolic`
- `battery-level-50-symbolic`
- `battery-level-40-symbolic`
- `battery-level-30-symbolic`
- `battery-level-20-symbolic`
- `battery-level-10-symbolic`
- `battery-level-0-symbolic`
- `battery-level-90-charging-symbolic`
- `battery-level-80-charging-symbolic`

The icon mapping should live in a helper such as `battery_icon_name(snapshot: &BatterySnapshot) -> &'static str`. This keeps the policy easy to inspect and later customize.

## Rendering Rules

The visible battery item renders three pieces:

- Icon: derived from `state` and rounded battery percentage.
- Percentage: formatted as `{percentage:.0}%`.
- Energy rate: formatted directly from `energy_rate` as watts, for example `6.2W`.

`energy_rate` from `wayle-battery` is already measured in watts, so it must not be divided by 1000.

When battery data is unavailable, the icon uses `battery-missing-symbolic`, the percentage text remains `NaN`, and the energy rate text is blank.

## Styling

Add CSS classes for the visible bar content:

- `battery-content`
- `battery-icon`
- `battery-percentage`
- `battery-energy-rate`

The content should be compact and align with the existing bar item styling. The icon uses a fixed 16px size so icon changes do not resize the bar item.

## Testing And Verification

Verification should include:

- `cargo fmt`
- `cargo check`
- Manual runtime check that the bar item shows icon, percentage, and watts.
- Manual check that unplugging or changing charge state changes the icon when UPower reports the new state.

The highest-risk regression is stale watts display. The watcher must include `energy_rate.watch()` so energy rate changes refresh the snapshot.
