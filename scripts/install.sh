#!/usr/bin/env bash
# Orbiscreen - Installation Script (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

echo "[Orbiscreen] Installing Secondary Display..."

INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"

if command -v cargo >/dev/null 2>&1; then
    echo "[Orbiscreen] Building daemon..."
    cargo build --release -p orbiscreen-daemon
    cp target/release/orbiscreen ~/.local/bin/

    echo "[Orbiscreen] Installing desktop entry and icon..."
    mkdir -p ~/.local/share/applications
    mkdir -p ~/.local/share/icons/hicolor/scalable/apps
    cp data/com.orbiscreen.OrbiscreenGtk.desktop ~/.local/share/applications/
    cp data/orbiscreen.svg ~/.local/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg

    echo "[Orbiscreen] Installing web client files..."
    mkdir -p ~/.local/share/orbiscreen/client/vendor
    cp clients/web/index.html ~/.local/share/orbiscreen/client/
    cp clients/web/style.css ~/.local/share/orbiscreen/client/
    cp clients/web/app.js ~/.local/share/orbiscreen/client/
    cp -r clients/web/vendor/. ~/.local/share/orbiscreen/client/vendor/

    echo "[Orbiscreen] Binary installed to ${INSTALL_DIR}/orbiscreen"
else
    echo "[Orbiscreen] Error: Cargo not found. Please install Rust or download prebuilt release binary."
    exit 1
fi

SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"
mkdir -p "${SYSTEMD_USER_DIR}"

cat <<'EOF' > "${SYSTEMD_USER_DIR}/orbiscreen.service"
[Unit]
Description=Orbiscreen Secondary Display Daemon
After=network.target

[Service]
ExecStart=%h/.local/bin/orbiscreen start
NoNewPrivileges=true
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF

echo "[Orbiscreen] Installed systemd user unit to ${SYSTEMD_USER_DIR}/orbiscreen.service"
echo ""
echo "[Orbiscreen] Installation complete."
echo "You can now run 'orbiscreen start' to begin streaming,"
echo "or use the 'Orbiscreen' app icon in your app launcher."
echo ""
echo "To enable background autostart via systemd:"
echo "  systemctl --user daemon-reload"
echo "  systemctl --user enable --now orbiscreen"
echo "To uninstall later, run ./scripts/uninstall.sh"
