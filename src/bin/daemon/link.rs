//! BlueZ link + state refresher.
//!
//! Polls BlueZ via `bluetoothctl` on a slow cadence to keep the cached
//! `DeviceState` in sync with reality — connection state, name, address,
//! capabilities, trust, RSSI. Audio side gets refreshed at the same time
//! out of PipeWire.
//!
//! The proper AAP/L2CAP loop will live alongside this once the byte
//! captures land; until then battery/in-ear/buttons show no live data
//! when the daemon is up (the rest of the snapshot is real).

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::time::sleep;
use tracing::{debug, info};

use podctl::{Capabilities, Event, Response, audio, bluez, caps::Model, model::PairedDevice};

use super::{Daemon, aap::AapTask};

const POLL_INTERVAL: Duration = Duration::from_secs(3);

pub async fn run(daemon: Arc<Daemon>) {
    info!(
        "link task: polling BlueZ + PipeWire every {:?}",
        POLL_INTERVAL
    );
    let mut prev_addr: Option<String> = None;
    let mut prev_connected = false;
    let mut aap_task: Option<AapTask> = None;
    loop {
        // BlueZ + PipeWire calls are sync subprocess work; park them on
        // a blocking thread so we don't stall the runtime if pactl is slow.
        let refresh = tokio::task::spawn_blocking(snapshot_now).await.ok();
        if let Some(snap) = refresh {
            let prev_addr_for_event = prev_addr.clone();
            let snap_addr = snap.address.clone();
            let snap_connected = snap.connected;
            let snap_name = snap.name.clone();
            // Preserve cached AAP-derived fields (battery / in_ear) — the
            // BlueZ snapshot doesn't know about them.
            {
                let mut s = daemon.state.write().await;
                let cached_battery = s.battery;
                let cached_in_ear = s.in_ear;
                let cached_settings = s.settings.clone();
                let cached_caps = s.capabilities;
                let had_model = cached_caps.model != Model::Unknown;
                *s = snap;
                s.battery = cached_battery;
                s.in_ear = cached_in_ear;
                s.settings = cached_settings;
                // Capabilities are sticky: one bad `bluetoothctl` poll
                // returns no device → Unknown caps, which would make the
                // daemon reject set_mode/set_conv (require_caps) and the
                // UI show "unknown model". Keep the resolved model until
                // a *different* known one replaces it. (A real unpair
                // leaves a stale label until restart — acceptable vs the
                // flapping this prevents.)
                if had_model && s.capabilities.model == Model::Unknown {
                    s.capabilities = cached_caps;
                }
                Daemon::touch(&mut s);
                if let Some(addr) = &s.address {
                    if prev_addr.as_deref() != Some(addr.as_str()) || s.connected != prev_connected
                    {
                        prev_addr = Some(addr.clone());
                        prev_connected = s.connected;
                        if s.connected {
                            let _ = daemon.events.send(Event::Connected {
                                name: s.name.clone().unwrap_or_default(),
                                address: addr.clone(),
                            });
                        } else if prev_addr_for_event.is_some() {
                            let _ = daemon.events.send(Event::Disconnected);
                        }
                    }
                } else if prev_addr.is_some() {
                    prev_addr = None;
                    prev_connected = false;
                    let _ = daemon.events.send(Event::Disconnected);
                }
            }
            // Spawn or stop the AAP loop based on the connection edge.
            sync_aap(
                &daemon,
                &mut aap_task,
                &snap_addr,
                snap_connected,
                snap_name,
            )
            .await;
        }
        sleep(POLL_INTERVAL).await;
    }
}

async fn sync_aap(
    daemon: &Arc<Daemon>,
    slot: &mut Option<AapTask>,
    addr: &Option<String>,
    connected: bool,
    _name: Option<String>,
) {
    match (addr, connected) {
        (Some(mac), true) if slot.is_none() => {
            // BlueZ may report "Connected" before the L2CAP server is ready.
            // Sleep briefly to let the device finish bringing up profiles.
            tokio::time::sleep(Duration::from_millis(800)).await;
            let task = super::aap::spawn(daemon.clone(), mac.clone());
            *slot = Some(task);
        }
        (_, false) if slot.is_some() => {
            if let Some(task) = slot.take() {
                tokio::task::spawn_blocking(move || task.shutdown())
                    .await
                    .ok();
            }
            // Clear the AAP-derived state — values are stale once the link
            // drops.
            let mut s = daemon.state.write().await;
            s.battery = podctl::Battery::default();
            s.in_ear = podctl::InEar::default();
            Daemon::touch(&mut s);
        }
        _ => {}
    }
}

/// Build a full DeviceState by querying BlueZ + PipeWire right now.
/// Returns the default (everything empty) when no AirPods are paired.
fn snapshot_now() -> podctl::DeviceState {
    let mut s = podctl::DeviceState::default();
    if let Some(dev) = bluez::primary_airpods() {
        s.address = Some(dev.address.clone());
        s.name = Some(dev.name.clone());
        s.connected = dev.connected;
        s.capabilities = dev.capabilities();
        s.bluetooth = bluez::bt_state(&dev);
    } else {
        s.capabilities = Capabilities::default();
    }
    s.audio = audio::snapshot();
    s.updated_at = now_secs();
    debug!(
        connected = s.connected,
        addr = s.address.as_deref().map(redact_mac).unwrap_or_default(),
        "snapshot refreshed"
    );
    s
}

/// Apple-OUI only — keep the vendor visible in logs, drop the device-unique bytes.
fn redact_mac(mac: &str) -> String {
    let parts: Vec<&str> = mac.split(':').collect();
    if parts.len() == 6 {
        format!("{}:{}:{}:**:**:**", parts[0], parts[1], parts[2])
    } else {
        "<mac>".into()
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// Request-side helpers: daemon dispatch calls into these for BlueZ ops.
// They shell out via `podctl::bluez` and update cached state on success.

pub async fn bt_connect(daemon: &Daemon) -> Response {
    let Some(addr) = daemon.state.read().await.address.clone() else {
        return Response::err("no AirPods paired — run 'podctl pair' first.");
    };
    match tokio::task::spawn_blocking(move || bluez::connect(&addr)).await {
        Ok(Ok(())) => {
            let mut s = daemon.state.write().await;
            s.connected = true;
            Daemon::touch(&mut s);
            Response::ok_done()
        }
        Ok(Err(e)) => Response::err(format!("{e}")),
        Err(e) => Response::err(format!("connect task: {e}")),
    }
}

pub async fn bt_disconnect(daemon: &Daemon) -> Response {
    let Some(addr) = daemon.state.read().await.address.clone() else {
        return Response::err("no AirPods paired.");
    };
    match tokio::task::spawn_blocking(move || bluez::disconnect(&addr)).await {
        Ok(Ok(())) => {
            let mut s = daemon.state.write().await;
            s.connected = false;
            Daemon::touch(&mut s);
            let _ = daemon.events.send(Event::Disconnected);
            Response::ok_done()
        }
        Ok(Err(e)) => Response::err(format!("{e}")),
        Err(e) => Response::err(format!("disconnect task: {e}")),
    }
}

pub async fn bt_pair(_daemon: &Daemon) -> Response {
    let res = tokio::task::spawn_blocking(|| {
        let found = bluez::discover(12)?;
        let target = found.into_iter().find(|d| !d.paired).ok_or_else(|| {
            anyhow::anyhow!("no new AirPods spotted — open the case until the LED blinks white.")
        })?;
        bluez::pair(&target.address)?;
        Ok::<_, anyhow::Error>(())
    })
    .await;
    match res {
        Ok(Ok(())) => Response::ok_done(),
        Ok(Err(e)) => Response::err(format!("{e}")),
        Err(e) => Response::err(format!("pair task: {e}")),
    }
}

pub async fn bt_unpair(daemon: &Daemon) -> Response {
    let Some(addr) = daemon.state.read().await.address.clone() else {
        return Response::err("nothing to unpair.");
    };
    match tokio::task::spawn_blocking(move || bluez::unpair(&addr)).await {
        Ok(Ok(())) => {
            let mut s = daemon.state.write().await;
            *s = podctl::DeviceState::default();
            Daemon::touch(&mut s);
            Response::ok_done()
        }
        Ok(Err(e)) => Response::err(format!("{e}")),
        Err(e) => Response::err(format!("unpair task: {e}")),
    }
}

pub async fn bt_list() -> Response {
    let res = tokio::task::spawn_blocking(|| {
        bluez::paired_airpods().map(|v| {
            v.iter()
                .map(bluez::to_paired_device)
                .collect::<Vec<PairedDevice>>()
        })
    })
    .await;
    match res {
        Ok(Ok(items)) => Response::ok_list(items),
        Ok(Err(e)) => Response::err(format!("{e}")),
        Err(e) => Response::err(format!("list task: {e}")),
    }
}

pub async fn bt_set_trusted(daemon: &Daemon, on: bool) -> Response {
    let Some(addr) = daemon.state.read().await.address.clone() else {
        return Response::err("no AirPods paired.");
    };
    match tokio::task::spawn_blocking(move || bluez::set_trusted(&addr, on)).await {
        Ok(Ok(())) => {
            let mut s = daemon.state.write().await;
            s.bluetooth.trusted = on;
            s.bluetooth.auto_connect = on;
            Daemon::touch(&mut s);
            Response::ok_done()
        }
        Ok(Err(e)) => Response::err(format!("{e}")),
        Err(e) => Response::err(format!("trust task: {e}")),
    }
}
