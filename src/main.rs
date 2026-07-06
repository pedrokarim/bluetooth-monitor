mod autostart;
mod bluetooth;
mod config;
mod fonts;
mod theme;
mod tray;
mod ui;

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use eframe::egui;
use tokio::sync::mpsc;

use crate::bluetooth::{BluetoothCommand, DeviceInfo, DiscoveredDevice};
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub devices: Arc<Mutex<Vec<DeviceInfo>>>,
    pub last_error: Arc<Mutex<Option<String>>>,
    pub adapter_powered: Arc<AtomicBool>,
    pub adapter_name: Arc<Mutex<Option<String>>>,
    pub visible: Arc<AtomicBool>,
    pub quit: Arc<AtomicBool>,
    pub cmd_tx: mpsc::UnboundedSender<BluetoothCommand>,
    pub ctx: Arc<Mutex<Option<egui::Context>>>,
    pub config: Arc<Mutex<Config>>,
    pub nearby: Arc<Mutex<Vec<DiscoveredDevice>>>,
    pub scanning: Arc<AtomicBool>,
    pub scan_started: Arc<Mutex<Option<std::time::Instant>>>,
    pub last_refresh: Arc<Mutex<Option<std::time::Instant>>>,
    /// Sliding window of the last RSSI samples per device address.
    pub rssi_history: Arc<Mutex<HashMap<String, VecDeque<i16>>>>,
}

fn main() -> eframe::Result<()> {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

    let mut config = Config::load();
    // Reconcile config.autostart with the actual .desktop file on disk. The
    // filesystem wins — if the user removed the file manually, the toggle
    // should reflect that on next launch.
    let real_autostart = autostart::is_enabled();
    if config.autostart != real_autostart {
        config.autostart = real_autostart;
        config.save();
    }
    let initial_theme = config.theme;

    let state = AppState {
        devices: Arc::new(Mutex::new(Vec::new())),
        last_error: Arc::new(Mutex::new(None)),
        adapter_powered: Arc::new(AtomicBool::new(false)),
        adapter_name: Arc::new(Mutex::new(None)),
        visible: Arc::new(AtomicBool::new(true)),
        quit: Arc::new(AtomicBool::new(false)),
        cmd_tx,
        ctx: Arc::new(Mutex::new(None)),
        config: Arc::new(Mutex::new(config)),
        nearby: Arc::new(Mutex::new(Vec::new())),
        scanning: Arc::new(AtomicBool::new(false)),
        scan_started: Arc::new(Mutex::new(None)),
        last_refresh: Arc::new(Mutex::new(None)),
        rssi_history: Arc::new(Mutex::new(HashMap::new())),
    };

    let backend_state = state.clone();
    std::thread::Builder::new()
        .name("bt-backend".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(bluetooth::run(backend_state, cmd_rx));
        })
        .expect("spawn backend");

    tray::spawn(state.clone());

    let icon = theme::app_icon(64);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 740.0])
            .with_min_inner_size([620.0, 460.0])
            .with_title("Bluetooth Monitor")
            .with_icon(icon),
        ..Default::default()
    };

    let boot_state = state.clone();
    eframe::run_native(
        "Bluetooth Monitor",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_pixels_per_point(1.0);
            fonts::install(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let t = theme::Theme::from_kind(initial_theme);
            theme::apply_style(&cc.egui_ctx, &t);
            *boot_state.ctx.lock().unwrap() = Some(cc.egui_ctx.clone());
            let icons = ui::load_icons(&cc.egui_ctx);
            Ok(Box::new(App::new(boot_state, initial_theme, icons)) as Box<dyn eframe::App>)
        }),
    )
}

struct App {
    state: AppState,
    ui_state: ui::UiState,
    last_visible: bool,
    started_at: std::time::Instant,
}

impl App {
    fn new(state: AppState, theme_kind: theme::ThemeKind, icons: ui::Icons) -> Self {
        let mut ui_state = ui::UiState::new(theme_kind);
        ui_state.logo = Some(icons.logo.clone());
        ui_state.icons = Some(icons);
        Self {
            state,
            ui_state,
            last_visible: true,
            started_at: std::time::Instant::now(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state.quit.load(Ordering::Relaxed) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        let vis = self.state.visible.load(Ordering::Relaxed);
        if vis != self.last_visible {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(vis));
            if vis {
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            self.last_visible = vis;
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            let close_to_tray = self.state.config.lock().unwrap().close_to_tray;
            if close_to_tray {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.state.visible.store(false, Ordering::Relaxed);
                self.last_visible = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }

        // Splash screen for the first 1.8 seconds, or until devices are populated
        let elapsed = self.started_at.elapsed().as_secs_f32();
        let has_devices = !self.state.devices.lock().unwrap().is_empty();
        let show_splash = elapsed < 1.8 || (!has_devices && elapsed < 3.5);

        if show_splash {
            ctx.request_repaint_after(Duration::from_millis(40));
            ui::render_splash(ctx, &self.ui_state, elapsed);
        } else {
            ctx.request_repaint_after(Duration::from_millis(500));
            ui::render(ctx, &self.state, &mut self.ui_state);
        }
    }
}
