#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Uninstallation Script
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Environment & Directory ──
set -euo pipefail
cd "$(dirname "$0")/.."

echo "[Orbiscreen] Uninstalling..."

# ── Stop & Disable User Service ──
if systemctl --user is-active --quiet orbiscreen; then
    echo "[Orbiscreen] Stopping service..."
    systemctl --user stop orbiscreen || true
fi

if systemctl --user is-enabled --quiet orbiscreen; then
    echo "[Orbiscreen] Disabling service..."
    systemctl --user disable orbiscreen || true
fi
systemctl --user daemon-reload || true

# ── Remove User Files ──
echo "[Orbiscreen] Removing binary and service files..."
rm -f "$HOME/.local/bin/orbiscreen"
rm -f "$HOME/.local/bin/orbiscreen-gtk"
rm -f "$HOME/.config/systemd/user/orbiscreen.service"

rm -f "$HOME/.local/share/applications/orbiscreen.desktop"
rm -f "$HOME/.local/share/applications/com.orbiscreen.OrbiscreenGtk.desktop"
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/orbiscreen.svg"
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg"
rm -rf "$HOME/.local/share/orbiscreen"

# ── Remove System Files (if root) ──
if [ "$EUID" -eq 0 ]; then
    echo "[Orbiscreen] Removing system-wide files..."
    rm -f /usr/bin/orbiscreen
    rm -f /usr/bin/orbiscreen-gtk
    rm -f /usr/share/applications/orbiscreen.desktop
    rm -f /usr/share/applications/com.orbiscreen.OrbiscreenGtk.desktop
    rm -f /usr/share/icons/hicolor/scalable/apps/orbiscreen.svg
    rm -f /usr/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg
    rm -rf /usr/share/orbiscreen
fi

echo "[Orbiscreen] Uninstallation complete."
