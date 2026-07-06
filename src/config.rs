use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::theme::ThemeKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeKind,
    #[serde(default = "default_refresh")]
    pub refresh_interval_secs: u64,
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
    #[serde(default = "default_true")]
    pub low_battery_alert: bool,
    #[serde(default = "default_battery_threshold")]
    pub low_battery_threshold: u8,
    #[serde(default)]
    pub autostart: bool,
}

fn default_refresh() -> u64 { 3 }
fn default_true() -> bool { true }
fn default_battery_threshold() -> u8 { 20 }

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeKind::default(),
            refresh_interval_secs: default_refresh(),
            close_to_tray: true,
            low_battery_alert: true,
            low_battery_threshold: default_battery_threshold(),
            autostart: false,
        }
    }
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        let dir = dirs::config_dir()?.join("bt-monitor");
        Some(dir.join("config.toml"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::path() else { return Self::default() };
        let Ok(bytes) = std::fs::read_to_string(&path) else { return Self::default() };
        toml::from_str(&bytes).unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = Self::path() else { return };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(s) = toml::to_string_pretty(self) {
            let _ = std::fs::write(&path, s);
        }
    }
}
