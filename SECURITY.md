# Security

## Reporting a vulnerability

If you discover a security-related issue in Bluetooth Monitor, please **do not
open a public GitHub issue**. Instead, contact the maintainer privately:

- Open a private security advisory on GitHub:
  https://github.com/pedrokarim/bluetooth-monitor/security/advisories/new

A private disclosure gives me time to prepare a fix before the vulnerability
is public. I will confirm receipt within a few days and coordinate a
disclosure timeline with you.

## Scope

- Anything under this repository's `src/`, `assets/`, or build tooling
- The interaction between this app and BlueZ / D-Bus / ksni

Vulnerabilities in upstream dependencies (`bluer`, `eframe`, `ksni`, ...)
should be reported to those projects directly. Feel free to open a
non-security issue here to track upgrading once fixed upstream.
