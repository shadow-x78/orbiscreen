#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Local Installation Script
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Environment & Directory ──
set -euo pipefail
cd "$(dirname "$0")/.."

echo "[Orbiscreen] Installing Secondary Display..."
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"

# ── Build & Install Binary ──
if command -v cargo >/dev/null 2>&1; then
    echo "[Orbiscreen] Building daemon..."
    cargo build --release -p orbiscreen-daemon
    systemctl --user stop orbiscreen 2>/dev/null || true
    install -m755 target/release/orbiscreen "${INSTALL_DIR}/orbiscreen.new"
    mv -f "${INSTALL_DIR}/orbiscreen.new" "${INSTALL_DIR}/orbiscreen"

# ── Install Web Client Assets ──
    echo "[Orbiscreen] Installing web client files..."
    mkdir -p ~/.local/share/orbiscreen/client/vendor
    cp clients/web/index.html ~/.local/share/orbiscreen/client/
    cp clients/web/style.css ~/.local/share/orbiscreen/client/
    cp clients/web/app.js ~/.local/share/orbiscreen/client/
    cp clients/web/favicon.svg ~/.local/share/orbiscreen/client/
    cp clients/web/favicon.png ~/.local/share/orbiscreen/client/
    cp clients/web/apple-touch-icon.png ~/.local/share/orbiscreen/client/
    cp -r clients/web/vendor/. ~/.local/share/orbiscreen/client/vendor/

    echo "[Orbiscreen] Binary installed to ${INSTALL_DIR}/orbiscreen"
else
    echo "[Orbiscreen] Error: Cargo not found. Please install Rust or download prebuilt release binary."
    exit 1
fi

# ── Systemd User Service ──
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

# ── Final Instructions ──
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
