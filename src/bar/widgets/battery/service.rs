use crate::bar::state::{BarItemState, BatterySnapshot, BatteryState};
use crate::shell::ShellMsg;
use futures::{FutureExt, StreamExt, future, select};
use relm4::Sender;
use std::sync::Arc;
use wayle_battery::BatteryService;
use wayle_power_profiles::PowerProfilesService;
use wayle_power_profiles::types::profile::PowerProfile;

pub(super) fn start(
    sender: Sender<ShellMsg>,
    service: Option<Arc<BatteryService>>,
    power_profiles: Option<Arc<PowerProfilesService>>,
) -> relm4::tokio::task::JoinHandle<()> {
    relm4::spawn(async move {
        run_battery_watcher(sender, service, power_profiles).await;
    })
}

async fn run_battery_watcher(
    sender: Sender<ShellMsg>,
    service: Option<Arc<BatteryService>>,
    power_profiles: Option<Arc<PowerProfilesService>>,
) {
    let Some(service) = service else {
        let _ = sender.send(battery_message(BatteryState::Unavailable));
        return;
    };

    send_battery_snapshot(&sender, service.as_ref(), power_profiles.as_deref());

    let mut percentage_updates = service.device.percentage.watch().fuse();
    let mut state_updates = service.device.state.watch().fuse();
    let mut energy_rate_updates = service.device.energy_rate.watch().fuse();
    let mut active_profile_updates = power_profiles
        .as_ref()
        .map(|service| service.power_profiles.active_profile.watch().fuse());
    let mut available_profile_updates = power_profiles
        .as_ref()
        .map(|service| service.power_profiles.profiles.watch().fuse());

    loop {
        select! {
            update = percentage_updates.next() => {
                if update.is_none() {
                    break;
                }

                send_battery_snapshot(&sender, &service, power_profiles.as_deref());
            }
            update = state_updates.next() => {
                if update.is_none() {
                    break;
                }
                send_battery_snapshot(&sender, &service, power_profiles.as_deref());
            }
            update = energy_rate_updates.next() => {
                if update.is_none() {
                    break;
                }

                send_battery_snapshot(&sender, &service, power_profiles.as_deref());
            }
            update = async {
                match active_profile_updates.as_mut() {
                    Some(stream) => stream.next().await,
                    None => future::pending().await,
                }
            }.fuse() => {
                if update.is_none() {
                    active_profile_updates = None;
                }

                send_battery_snapshot(&sender, &service, power_profiles.as_deref());
            }

            update = async {
                match available_profile_updates.as_mut() {
                    Some(stream) => stream.next().await,
                    None => future::pending().await,
                }
            }.fuse() => {
                if update.is_none() {
                    available_profile_updates = None;
                }

                send_battery_snapshot(&sender, &service, power_profiles.as_deref());
            }
        }
    }

    let _ = sender.send(battery_message(BatteryState::Unavailable));
}

fn send_battery_snapshot(
    sender: &Sender<ShellMsg>,
    service: &BatteryService,
    power_profiles: Option<&PowerProfilesService>,
) {
    let percentage = service.device.percentage.get();
    let state = service.device.state.get();
    let energy_rate = service.device.energy_rate.get();

    let active_profile = power_profiles.map(|service| service.power_profiles.active_profile.get());

    let available_profiles = power_profiles
        .map(|service| {
            service
                .power_profiles
                .profiles
                .get()
                .into_iter()
                .map(|profile| profile.profile)
                .filter(|profile| *profile != PowerProfile::Unknown)
                .collect()
        })
        .unwrap_or_default();

    let snapshot = BatterySnapshot {
        percentage,
        state,
        energy_rate,
        active_profile,
        available_profiles,
    };

    let _ = sender.send(battery_message(BatteryState::Ready(snapshot)));
}

fn battery_message(state: BatteryState) -> ShellMsg {
    ShellMsg::ItemStateChanged(BarItemState::Battery(state))
}
