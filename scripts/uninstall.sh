#!/usr/bin/env bash
# Orbiscreen - uninstall script (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

echo "Uninstalling Orbiscreen..."

# Stop and disable systemd service
if systemctl --user is-active --quiet orbiscreen; then
    echo "Stopping Orbiscreen service..."
    systemctl --user stop orbiscreen || true
fi

if systemctl --user is-enabled --quiet orbiscreen; then
    echo "Disabling Orbiscreen service..."
    systemctl --user disable orbiscreen || true
fi
systemctl --user daemon-reload || true

# Remove user files
echo "Removing binary and service files..."
rm -f "$HOME/.local/bin/orbiscreen"
rm -f "$HOME/.config/systemd/user/orbiscreen.service"

# Remove desktop entry and icon if installed locally
rm -f "$HOME/.local/share/applications/com.orbiscreen.OrbiscreenGtk.desktop"
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg"

# Remove system-wide if run as root
if [ "$EUID" -eq 0 ]; then
    echo "Removing system-wide files..."
    rm -f /usr/bin/orbiscreen
    rm -f /usr/share/applications/com.orbiscreen.OrbiscreenGtk.desktop
    rm -f /usr/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg
fi

echo "✅ Orbiscreen has been completely uninstalled."
