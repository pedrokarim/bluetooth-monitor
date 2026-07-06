# Changelog

All notable changes to this project will be documented in this file. The
format is loosely based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

## [0.1.0] &mdash; 2026-07-06

Initial release.

### Added

- Live dashboard for known Bluetooth devices with battery percentage and RSSI
- Battery rings donut, one ring per connected device
- Device detail page (metrics, properties, services, actions)
- Discover screen with animated radar and BlueZ discovery streaming
- Settings page with instant theme switching
- Four themes with structural differences beyond palette:
  - Aurora (violet gradient, Inter Light hero, pill everything)
  - Nova (charcoal solid, Space Grotesk hero, rounded squares)
  - Command (terminal black, JetBrains Mono hero, sharp corners)
  - Radiant (cream gradient, Instrument Serif Italic hero, editorial)
- Splash screen with animated Bluetooth logo, sweep arc, pulse dots
- StatusNotifierItem tray icon with tooltip, menu, and window toggle
- Persistent configuration at `~/.config/bt-monitor/config.toml`
- Refresh interval selectable (1&nbsp;s / 3&nbsp;s / 5&nbsp;s / 10&nbsp;s / 30&nbsp;s), applied hot
- Close-to-tray, low battery alert, autostart preferences

[Unreleased]: https://github.com/pedrokarim/bluetooth-monitor/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/pedrokarim/bluetooth-monitor/releases/tag/v0.1.0
