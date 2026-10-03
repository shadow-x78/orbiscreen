#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - Debian/Ubuntu (.deb) Package Builder
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Environment & Directory ──
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION="${1:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')}"
# dpkg is authoritative; the uname fallback keeps the name valid when this script is
# inspected on a non-Debian host.
ARCH="$(dpkg --print-architecture 2>/dev/null || true)"
if [ -z "${ARCH}" ]; then
    case "$(uname -m)" in
        x86_64)  ARCH="amd64" ;;
        aarch64) ARCH="arm64" ;;
        *)       ARCH="$(uname -m)" ;;
    esac
fi
BUILD_DIR="target/deb-staging"
DEB_NAME="orbiscreen_${VERSION}_${ARCH}.deb"

echo "[Orbiscreen] Building Debian package for Orbiscreen v${VERSION} (${ARCH})..."

# ── Build Binaries ──
if [ ! -f target/release/orbiscreen ]; then
    echo "[Orbiscreen] Building release binaries for deb..."
    cargo build --release --workspace
fi

# ── Prepare Staging Layout ──
rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/lib/systemd/user"
mkdir -p "${BUILD_DIR}/usr/lib/udev/rules.d"
mkdir -p "${BUILD_DIR}/usr/share/orbiscreen/client"

cp -f target/release/orbiscreen "${BUILD_DIR}/usr/bin/"
if [ -f target/release/orbiscreen-gui ]; then
    cp -f target/release/orbiscreen-gui "${BUILD_DIR}/usr/bin/"
fi
mkdir -p "${BUILD_DIR}/usr/share/applications" "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps"
cp -f data/orbiscreen.desktop "${BUILD_DIR}/usr/share/applications/"
cp -f data/orbiscreen.svg "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/"
ln -sf orbiscreen.svg "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/orbiscreen-gui.svg"
for size in 16 24 32 48 64 128 256 512; do
    if [ -f "crates/orbiscreen-gui/icons/${size}x${size}.png" ]; then
        mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/${size}x${size}/apps"
        cp -f "crates/orbiscreen-gui/icons/${size}x${size}.png" "${BUILD_DIR}/usr/share/icons/hicolor/${size}x${size}/apps/orbiscreen.png"
        ln -sf orbiscreen.png "${BUILD_DIR}/usr/share/icons/hicolor/${size}x${size}/apps/orbiscreen-gui.png"
    fi
done

for f in index.html style.css app.js annexb.js stats.js favicon.svg favicon.png apple-touch-icon.png; do
    cp -f "clients/web/${f}" "${BUILD_DIR}/usr/share/orbiscreen/client/"
done
cp -f scripts/install-evdi-module.sh "${BUILD_DIR}/usr/share/orbiscreen/"
cp -f data/99-orbiscreen-usb.rules "${BUILD_DIR}/usr/lib/udev/rules.d/"

# ── Service Definition ──
cat << 'EOF' > "${BUILD_DIR}/usr/lib/systemd/user/orbiscreen.service"
[Unit]
Description=Orbiscreen Virtual Secondary Display Service
Documentation=https://github.com/shadow-x78/orbiscreen
After=graphical-session.target

[Service]
Type=exec
ExecStart=/usr/bin/orbiscreen start
NoNewPrivileges=true
Restart=on-failure
RestartSec=3s

[Install]
WantedBy=graphical-session.target
EOF

# ── Debian Control Metadata ──
cat << EOF > "${BUILD_DIR}/DEBIAN/control"
Package: orbiscreen
Version: ${VERSION}
Architecture: ${ARCH}
Maintainer: shadow-x78 <shadow-x78@users.noreply.github.com>
Depends: libgstreamer1.0-0, libgstreamer-plugins-base1.0-0, gstreamer1.0-plugins-good, gstreamer1.0-plugins-bad, gstreamer1.0-plugins-ugly, gstreamer1.0-libav, libxkbcommon0, libevdev2
Recommends: android-tools-adb
Suggests: evdi-dkms
Section: utils
Priority: optional
Homepage: https://github.com/shadow-x78/orbiscreen
Description: Turn any Android tablet or phone into a second monitor for Linux
 Orbiscreen turns Android tablets and phones into low-latency
 extended displays for Linux desktops on Wayland and X11. Features native
 graphic tablet digitizer with stylus pressure and tilt for Krita/GIMP,
 auto-orientation, and hardware encoding (NVENC/VAAPI).
EOF

# ── Post-Install Script ──
cat <<'EOF' > "${BUILD_DIR}/DEBIAN/postinst"
set -e
if [ -x /usr/bin/gtk-update-icon-cache ]; then
    /usr/bin/gtk-update-icon-cache /usr/share/icons/hicolor >/dev/null 2>&1 || true
fi
if [ -x /usr/bin/update-desktop-database ]; then
    /usr/bin/update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
fi
if [ -x /usr/bin/udevadm ]; then
    /usr/bin/udevadm control --reload >/dev/null 2>&1 || true
    /usr/bin/udevadm trigger >/dev/null 2>&1 || true
fi
# The unit is WantedBy=graphical-session.target, so enable it for every session that
# currently has one; without this the installed daemon never starts on its own.
for u in $(users | tr ' ' '\n' | tail -n +2 | sort -u); do
    su -s /bin/sh -c "systemctl --user daemon-reload && systemctl --user enable orbiscreen.service" "$u" >/dev/null 2>&1 || true
done
exit 0
EOF
chmod +x "${BUILD_DIR}/DEBIAN/postinst"

# ── Pre-Removal Script ──
cat <<'EOF' > "${BUILD_DIR}/DEBIAN/prerm"
set -e
if [ "$1" = "remove" ] || [ "$1" = "deconfigure" ]; then
    for u in $(users | tr ' ' '\n' | tail -n +2 | sort -u); do
        su -s /bin/sh -c "systemctl --user disable --now orbiscreen.service || true" "$u" || true
    done
fi
exit 0
EOF
chmod +x "${BUILD_DIR}/DEBIAN/prerm"

# ── Post-Removal Script ──
cat <<'EOF' > "${BUILD_DIR}/DEBIAN/postrm"
set -e
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    if [ -x /usr/bin/gtk-update-icon-cache ]; then
        /usr/bin/gtk-update-icon-cache /usr/share/icons/hicolor >/dev/null 2>&1 || true
    fi
    if [ -x /usr/bin/update-desktop-database ]; then
        /usr/bin/update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
    fi
    echo "[Orbiscreen] Orbiscreen has been removed."
fi
exit 0
EOF
chmod +x "${BUILD_DIR}/DEBIAN/postrm"

# ── Build Final Package ──
dpkg-deb --build "${BUILD_DIR}" "${DEB_NAME}"
echo "[Orbiscreen] Debian package built successfully: ${DEB_NAME}"
