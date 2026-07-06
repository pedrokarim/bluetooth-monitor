#!/usr/bin/env bash
# User-space installer for Bluetooth Monitor.
# Installs to ~/.local — no sudo required.
#
# Usage:
#   ./install.sh                # build (if needed) and install
#   ./install.sh --no-build     # skip cargo build, use existing binary
#   ./install.sh --uninstall    # remove all installed files

set -euo pipefail

APP_NAME="bluetooth-monitor"
APP_DISPLAY="Bluetooth Monitor"

BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
APPS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICONS_BASE="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DESKTOP_SRC="${SCRIPT_DIR}/packaging/${APP_NAME}.desktop"
ICON_SRC="${SCRIPT_DIR}/assets/bt-logo.png"

# ── flags ───────────────────────────────────────────────────────────────────
DO_BUILD=1
DO_UNINSTALL=0
for arg in "$@"; do
  case "$arg" in
    --no-build)  DO_BUILD=0 ;;
    --uninstall) DO_UNINSTALL=1 ;;
    -h|--help)
      sed -n '1,10p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "unknown flag: $arg" >&2
      exit 2
      ;;
  esac
done

info()  { printf "\033[1;36m::\033[0m %s\n" "$*"; }
ok()    { printf "\033[1;32m✓\033[0m  %s\n" "$*"; }
warn()  { printf "\033[1;33m!\033[0m  %s\n" "$*"; }
fatal() { printf "\033[1;31m✗\033[0m  %s\n" "$*" >&2; exit 1; }

# ── uninstall ───────────────────────────────────────────────────────────────
uninstall() {
  info "Removing installed files"
  rm -f "${BIN_DIR}/${APP_NAME}"                                      && ok "removed ${BIN_DIR}/${APP_NAME}"        || true
  rm -f "${APPS_DIR}/${APP_NAME}.desktop"                             && ok "removed ${APPS_DIR}/${APP_NAME}.desktop" || true
  rm -f "${ICONS_BASE}/512x512/apps/${APP_NAME}.png"                  && ok "removed 512x512 icon"                    || true
  rm -f "${HOME}/.config/autostart/${APP_NAME}.desktop"               && ok "removed autostart entry"                 || true
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${APPS_DIR}" >/dev/null 2>&1 || true
  fi
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${ICONS_BASE}" >/dev/null 2>&1 || true
  fi
  ok "Uninstall complete."
  echo
  info "Config was left in place at ~/.config/bt-monitor/"
  info "Remove it manually if you want a clean slate."
  exit 0
}

if [[ "${DO_UNINSTALL}" == 1 ]]; then
  uninstall
fi

# ── build ───────────────────────────────────────────────────────────────────
if [[ "${DO_BUILD}" == 1 ]]; then
  info "Building release binary"
  ( cd "${SCRIPT_DIR}" && cargo build --release --locked )
  ok "Build finished"
fi

BIN_SRC="${SCRIPT_DIR}/target/release/${APP_NAME}"
[[ -x "${BIN_SRC}" ]] || fatal "binary not found at ${BIN_SRC} — run without --no-build or build manually"
[[ -f "${DESKTOP_SRC}" ]] || fatal ".desktop template missing at ${DESKTOP_SRC}"
[[ -f "${ICON_SRC}" ]] || fatal "icon missing at ${ICON_SRC}"

# ── install ─────────────────────────────────────────────────────────────────
mkdir -p "${BIN_DIR}" "${APPS_DIR}" "${ICONS_BASE}/512x512/apps"

info "Installing binary → ${BIN_DIR}/${APP_NAME}"
install -m 0755 "${BIN_SRC}" "${BIN_DIR}/${APP_NAME}"
ok "installed"

info "Installing .desktop → ${APPS_DIR}/"
install -m 0644 "${DESKTOP_SRC}" "${APPS_DIR}/${APP_NAME}.desktop"
ok "installed"

info "Installing icon → ${ICONS_BASE}/512x512/apps/"
install -m 0644 "${ICON_SRC}" "${ICONS_BASE}/512x512/apps/${APP_NAME}.png"
ok "installed"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${APPS_DIR}" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "${ICONS_BASE}" >/dev/null 2>&1 || true
fi

echo
ok "${APP_DISPLAY} is installed."

# PATH check
if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
  warn "${BIN_DIR} is not in \$PATH"
  echo "   Add this to your shell rc (~/.bashrc or ~/.zshrc):"
  echo "     export PATH=\"${BIN_DIR}:\$PATH\""
  echo "   Then reopen your shell — or just run:"
  echo "     ${BIN_DIR}/${APP_NAME}"
else
  echo "   Launch by name:  ${APP_NAME}"
fi
echo "   Or from your desktop menu: search “${APP_DISPLAY}”."
