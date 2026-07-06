use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use bluer::{Adapter, Address};
use tokio::sync::mpsc;

use crate::AppState;

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub address: String,
    pub connected: bool,
    pub paired: bool,
    pub trusted: bool,
    pub blocked: bool,
    pub battery: Option<u8>,
    pub rssi: Option<i16>,
    pub tx_power: Option<i16>,
    pub icon: Option<String>,
    pub uuids: Vec<String>,
    pub manufacturer: Option<String>,
    pub class_of_device: Option<u32>,
}

#[derive(Debug)]
pub enum BluetoothCommand {
    Refresh,
    Connect(String),
    Disconnect(String),
    SetTrusted(String, bool),
    SetBlocked(String, bool),
    Remove(String),
    StartDiscovery,
    StopDiscovery,
    Pair(String),
}

#[derive(Clone, Debug)]
pub struct DiscoveredDevice {
    pub address: String,
    pub name: Option<String>,
    pub rssi: Option<i16>,
    pub seen_at: std::time::Instant,
}

pub async fn run(state: AppState, mut cmd_rx: mpsc::UnboundedReceiver<BluetoothCommand>) {
    let session = match bluer::Session::new().await {
        Ok(s) => s,
        Err(e) => {
            set_err(&state, format!("BlueZ session: {e}"));
            return;
        }
    };

    let adapter = match session.default_adapter().await {
        Ok(a) => a,
        Err(e) => {
            set_err(&state, format!("No Bluetooth adapter: {e}"));
            return;
        }
    };

    let mut discovery_handle: Option<tokio::task::JoinHandle<()>> = None;

    *state.adapter_name.lock().unwrap() = Some(adapter.name().to_string());

    if let Err(e) = adapter.set_powered(true).await {
        set_err(&state, format!("Power on: {e}"));
    }

    refresh(&state, &adapter).await;

    let mut last_secs = state.config.lock().unwrap().refresh_interval_secs.max(1);
    let mut interval = tokio::time::interval(Duration::from_secs(last_secs));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        // Reset the interval if the user changed it in Settings.
        let current_secs = state.config.lock().unwrap().refresh_interval_secs.max(1);
        if current_secs != last_secs {
            interval = tokio::time::interval(Duration::from_secs(current_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            last_secs = current_secs;
        }

        tokio::select! {
            _ = interval.tick() => {
                refresh(&state, &adapter).await;
                // Prune stale nearby entries (not seen for 15s)
                {
                    let mut nearby = state.nearby.lock().unwrap();
                    let now = std::time::Instant::now();
                    nearby.retain(|d| now.duration_since(d.seen_at).as_secs() < 15);
                }
            }
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    BluetoothCommand::StartDiscovery => {
                        if discovery_handle.is_none() {
                            let handle = spawn_discovery(state.clone(), adapter.clone());
                            discovery_handle = Some(handle);
                            state.scanning.store(true, Ordering::Relaxed);
                            *state.scan_started.lock().unwrap() = Some(std::time::Instant::now());
                        }
                    }
                    BluetoothCommand::StopDiscovery => {
                        if let Some(h) = discovery_handle.take() {
                            h.abort();
                        }
                        state.scanning.store(false, Ordering::Relaxed);
                        state.nearby.lock().unwrap().clear();
                    }
                    other => {
                        if let Err(e) = handle_command(&adapter, other).await {
                            set_err(&state, format!("Command failed: {e}"));
                        }
                        refresh(&state, &adapter).await;
                    }
                }
            }
        }
    }
}

fn spawn_discovery(state: AppState, adapter: Adapter) -> tokio::task::JoinHandle<()> {
    use futures::stream::StreamExt;
    tokio::spawn(async move {
        let discover = match adapter.discover_devices().await {
            Ok(d) => d,
            Err(e) => {
                set_err(&state, format!("Discovery: {e}"));
                state.scanning.store(false, Ordering::Relaxed);
                return;
            }
        };
        futures::pin_mut!(discover);
        while let Some(event) = discover.next().await {
            use bluer::AdapterEvent::*;
            match event {
                DeviceAdded(addr) => {
                    if let Ok(dev) = adapter.device(addr) {
                        let name = dev.name().await.ok().flatten();
                        let rssi = dev.rssi().await.ok().flatten();
                        let discovered = DiscoveredDevice {
                            address: addr.to_string(),
                            name,
                            rssi,
                            seen_at: std::time::Instant::now(),
                        };
                        let mut nearby = state.nearby.lock().unwrap();
                        nearby.retain(|d| d.address != discovered.address);
                        nearby.push(discovered);
                        nearby.sort_by(|a, b| b.rssi.cmp(&a.rssi));
                        if let Some(ctx) = state.ctx.lock().unwrap().as_ref() {
                            ctx.request_repaint();
                        }
                    }
                }
                DeviceRemoved(addr) => {
                    let mut nearby = state.nearby.lock().unwrap();
                    nearby.retain(|d| d.address != addr.to_string());
                }
                _ => {}
            }
        }
    })
}

const RSSI_HISTORY_MAX: usize = 60;

async fn refresh(state: &AppState, adapter: &Adapter) {
    *state.last_refresh.lock().unwrap() = Some(std::time::Instant::now());
    match collect_devices(adapter).await {
        Ok(list) => {
            // Append this refresh's RSSI reading to each device's rolling history
            // (only for connected devices — disconnected devices report None RSSI).
            {
                let mut hist = state.rssi_history.lock().unwrap();
                for d in &list {
                    if d.connected {
                        let entry = hist
                            .entry(d.address.clone())
                            .or_insert_with(|| std::collections::VecDeque::with_capacity(RSSI_HISTORY_MAX));
                        // Push either the RSSI or the previous value (so gaps carry over)
                        if let Some(rssi) = d.rssi {
                            entry.push_back(rssi);
                        } else if let Some(&last) = entry.back() {
                            entry.push_back(last);
                        }
                        while entry.len() > RSSI_HISTORY_MAX {
                            entry.pop_front();
                        }
                    }
                }
            }
            *state.devices.lock().unwrap() = list;
            *state.last_error.lock().unwrap() = None;
            let powered = adapter.is_powered().await.unwrap_or(false);
            state.adapter_powered.store(powered, Ordering::Relaxed);
            if let Some(ctx) = state.ctx.lock().unwrap().as_ref() {
                ctx.request_repaint();
            }
        }
        Err(e) => set_err(state, format!("Collect: {e}")),
    }
}

fn set_err(state: &AppState, msg: String) {
    *state.last_error.lock().unwrap() = Some(msg);
    if let Some(ctx) = state.ctx.lock().unwrap().as_ref() {
        ctx.request_repaint();
    }
}

async fn collect_devices(adapter: &Adapter) -> Result<Vec<DeviceInfo>> {
    let addresses = adapter.device_addresses().await?;
    let mut result = Vec::with_capacity(addresses.len());
    for addr in addresses {
        let d = match adapter.device(addr) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let name = d
            .name()
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| addr.to_string());
        let connected = d.is_connected().await.unwrap_or(false);
        let paired = d.is_paired().await.unwrap_or(false);
        let trusted = d.is_trusted().await.unwrap_or(false);
        let blocked = d.is_blocked().await.unwrap_or(false);
        let battery = d.battery_percentage().await.ok().flatten();
        let rssi = d.rssi().await.ok().flatten();
        let tx_power = d.tx_power().await.ok().flatten();
        let icon = d.icon().await.ok().flatten();
        let uuids = d
            .uuids()
            .await
            .ok()
            .flatten()
            .map(|s| s.into_iter().map(|u| u.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();
        let manufacturer = d
            .manufacturer_data()
            .await
            .ok()
            .flatten()
            .and_then(|m| m.keys().copied().next())
            .map(|id| format!("0x{id:04X}"));
        let class_of_device = d.class().await.ok().flatten();

        result.push(DeviceInfo {
            name,
            address: addr.to_string(),
            connected,
            paired,
            trusted,
            blocked,
            battery,
            rssi,
            tx_power,
            icon,
            uuids,
            manufacturer,
            class_of_device,
        });
    }
    result.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then(b.paired.cmp(&a.paired))
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(result)
}

async fn handle_command(adapter: &Adapter, cmd: BluetoothCommand) -> Result<()> {
    match cmd {
        BluetoothCommand::Refresh => {}
        BluetoothCommand::Connect(addr) => {
            let addr: Address = addr.parse()?;
            adapter.device(addr)?.connect().await?;
        }
        BluetoothCommand::Disconnect(addr) => {
            let addr: Address = addr.parse()?;
            adapter.device(addr)?.disconnect().await?;
        }
        BluetoothCommand::SetTrusted(addr, v) => {
            let addr: Address = addr.parse()?;
            adapter.device(addr)?.set_trusted(v).await?;
        }
        BluetoothCommand::SetBlocked(addr, v) => {
            let addr: Address = addr.parse()?;
            adapter.device(addr)?.set_blocked(v).await?;
        }
        BluetoothCommand::Remove(addr) => {
            let addr: Address = addr.parse()?;
            adapter.remove_device(addr).await?;
        }
        BluetoothCommand::Pair(addr) => {
            let addr: Address = addr.parse()?;
            adapter.device(addr)?.pair().await?;
        }
        BluetoothCommand::StartDiscovery | BluetoothCommand::StopDiscovery => {}
    }
    Ok(())
}
