#!/usr/bin/env bash
# Orbiscreen - Installation Script (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

echo "========================================="
echo " Installing Orbiscreen Secondary Display "
echo "========================================="

INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"

if command -v cargo >/dev/null 2>&1; then
    echo "--> Building Orbiscreen daemon..."
    cargo build --release -p orbiscreen-daemon
    cp target/release/orbiscreen "${INSTALL_DIR}/orbiscreen"
    echo "--> Binary installed to ${INSTALL_DIR}/orbiscreen"
else
    echo "--> Cargo not found. Please install Rust or download prebuilt release binary."
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
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF

echo "--> Installed systemd user unit to ${SYSTEMD_USER_DIR}/orbiscreen.service"
echo ""
echo "To start Orbiscreen:"
echo "  orbiscreen start"
echo ""
echo "To enable background autostart via systemd:"
echo "  systemctl --user daemon-reload"
echo "  systemctl --user enable --now orbiscreen"
echo "========================================="
