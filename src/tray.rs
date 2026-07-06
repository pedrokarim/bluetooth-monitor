use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::bluetooth::BluetoothCommand;
use crate::theme;
use crate::AppState;

struct BluetoothTray {
    state: AppState,
}

impl ksni::Tray for BluetoothTray {
    fn id(&self) -> String {
        "bluetooth-monitor".into()
    }

    fn title(&self) -> String {
        "Bluetooth Monitor".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let sizes = [22usize, 32, 48];
        sizes
            .into_iter()
            .map(|s| ksni::Icon {
                width: s as i32,
                height: s as i32,
                data: theme::tray_icon_argb(s),
            })
            .collect()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let devices = self.state.devices.lock().unwrap();
        let connected: Vec<_> = devices.iter().filter(|d| d.connected).cloned().collect();
        let title = if connected.is_empty() {
            "Bluetooth Monitor — no devices connected".into()
        } else {
            format!("Bluetooth Monitor — {} connected", connected.len())
        };
        let mut description = String::new();
        for d in &connected {
            if !description.is_empty() {
                description.push('\n');
            }
            match d.battery {
                Some(b) => description.push_str(&format!("• {} — {}%", d.name, b)),
                None => description.push_str(&format!("• {}", d.name)),
            }
        }
        ksni::ToolTip {
            icon_name: String::new(),
            icon_pixmap: vec![],
            title,
            description,
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let new = !self.state.visible.load(Ordering::Relaxed);
        self.state.visible.store(new, Ordering::Relaxed);
        self.request_repaint();
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        let _ = self.state.cmd_tx.send(BluetoothCommand::Refresh);
        self.request_repaint();
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let devices = self.state.devices.lock().unwrap().clone();
        let connected: Vec<_> = devices.iter().filter(|d| d.connected).cloned().collect();
        let mut items: Vec<ksni::MenuItem<Self>> = Vec::new();

        items.push(
            StandardItem {
                label: format!("Connected ({})", connected.len()),
                enabled: false,
                ..Default::default()
            }
            .into(),
        );

        if connected.is_empty() {
            items.push(
                StandardItem {
                    label: "  (none)".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        } else {
            for d in &connected {
                let label = match d.battery {
                    Some(b) => format!("  {} — {}%", d.name, b),
                    None => format!("  {}", d.name),
                };
                let addr = d.address.clone();
                items.push(
                    StandardItem {
                        label,
                        activate: Box::new(move |tray: &mut Self| {
                            let _ = tray
                                .state
                                .cmd_tx
                                .send(BluetoothCommand::Disconnect(addr.clone()));
                        }),
                        ..Default::default()
                    }
                    .into(),
                );
            }
        }

        let paired_disconnected: Vec<_> = devices
            .iter()
            .filter(|d| d.paired && !d.connected)
            .cloned()
            .collect();
        if !paired_disconnected.is_empty() {
            items.push(MenuItem::Separator);
            items.push(
                StandardItem {
                    label: format!("Reconnect ({})", paired_disconnected.len()),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
            for d in &paired_disconnected {
                let addr = d.address.clone();
                let name = d.name.clone();
                items.push(
                    StandardItem {
                        label: format!("  {}", name),
                        activate: Box::new(move |tray: &mut Self| {
                            let _ = tray
                                .state
                                .cmd_tx
                                .send(BluetoothCommand::Connect(addr.clone()));
                        }),
                        ..Default::default()
                    }
                    .into(),
                );
            }
        }

        items.push(MenuItem::Separator);

        items.push(
            StandardItem {
                label: "Show / hide window".into(),
                activate: Box::new(|tray: &mut Self| {
                    let new = !tray.state.visible.load(Ordering::Relaxed);
                    tray.state.visible.store(new, Ordering::Relaxed);
                    tray.request_repaint();
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(
            StandardItem {
                label: "Refresh".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.state.cmd_tx.send(BluetoothCommand::Refresh);
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        items.push(
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|tray: &mut Self| {
                    tray.state.quit.store(true, Ordering::Relaxed);
                    tray.request_repaint();
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

impl BluetoothTray {
    fn request_repaint(&self) {
        if let Some(ctx) = self.state.ctx.lock().unwrap().as_ref() {
            ctx.request_repaint();
        }
    }
}

pub fn spawn(state: AppState) {
    let tray = BluetoothTray {
        state: state.clone(),
    };
    let service = ksni::TrayService::new(tray);
    let handle = service.handle();
    service.spawn();

    std::thread::Builder::new()
        .name("bt-tray-refresh".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(3));
            handle.update(|_tray: &mut BluetoothTray| {});
        })
        .expect("spawn tray refresh");
}
