//! Autostart integration via XDG Desktop Entry.
//!
//! Writes / removes `~/.config/autostart/bluetooth-monitor.desktop` so the
//! app launches when the user's desktop session starts. The file honours
//! `Hidden=false` + `X-GNOME-Autostart-enabled=true` so GNOME's tools don't
//! silently drop it.

use std::path::PathBuf;

pub const APP_ID: &str = "bluetooth-monitor";

fn autostart_dir() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("autostart"))
}

fn autostart_file() -> Option<PathBuf> {
    Some(autostart_dir()?.join(format!("{APP_ID}.desktop")))
}

pub fn is_enabled() -> bool {
    autostart_file().map(|p| p.exists()).unwrap_or(false)
}

/// Enable or disable autostart. Idempotent: turning something on twice or off
/// twice is a no-op with an `Ok`.
pub fn set(enabled: bool) -> std::io::Result<()> {
    let Some(dir) = autostart_dir() else {
        return Ok(());
    };
    let path = dir.join(format!("{APP_ID}.desktop"));

    if !enabled {
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        return Ok(());
    }

    std::fs::create_dir_all(&dir)?;

    // Prefer running the installed command from PATH; fall back to the
    // absolute path of the current binary so autostart still works when the
    // user hasn't run `./install.sh` (e.g. hacking on a debug build).
    let exec = which_or_current("bluetooth-monitor")
        .or_else(|| std::env::current_exe().ok())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "bluetooth-monitor".to_string());

    let contents = format!(
        "[Desktop Entry]
Type=Application
Version=1.0
Name=Bluetooth Monitor
Comment=Live dashboard for Bluetooth devices
Exec={exec}
Icon=bluetooth-monitor
Terminal=false
Hidden=false
X-GNOME-Autostart-enabled=true
StartupNotify=true
StartupWMClass=bluetooth-monitor
Categories=Network;System;Utility;
"
    );

    std::fs::write(&path, contents)?;
    Ok(())
}

fn which_or_current(cmd: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(cmd);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
