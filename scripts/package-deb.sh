#!/usr/bin/env bash
# Orbiscreen - Debian/Ubuntu (.deb) Package Builder
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

VERSION="${1:-0.6.0}"
ARCH="amd64"
BUILD_DIR="target/deb-staging"
DEB_NAME="orbiscreen_${VERSION}_${ARCH}.deb"

echo "==> Building Debian package for Orbiscreen v${VERSION} (${ARCH})..."

mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/share/applications"
mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${BUILD_DIR}/usr/lib/systemd/user"

cp -f target/release/orbiscreen "${BUILD_DIR}/usr/bin/"
cp -f target/release/orbiscreen-daemon "${BUILD_DIR}/usr/bin/" || true
cp -f target/release/orbiscreen-gtk "${BUILD_DIR}/usr/bin/" || true

cp -f data/com.orbiscreen.OrbiscreenGtk.desktop "${BUILD_DIR}/usr/share/applications/" || true
cp -f data/orbiscreen.svg "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg" || true

cat << 'EOF' > "${BUILD_DIR}/usr/lib/systemd/user/orbiscreen.service"
[Unit]
Description=Orbiscreen Virtual Secondary Display Service
Documentation=https://github.com/shadow-x78/orbiscreen
After=graphical-session.target

[Service]
Type=exec
ExecStart=/usr/bin/orbiscreen-daemon
Restart=on-failure
RestartSec=3s

[Install]
WantedBy=graphical-session.target
EOF

cat << EOF > "${BUILD_DIR}/DEBIAN/control"
Package: orbiscreen
Version: ${VERSION}
Architecture: ${ARCH}
Maintainer: shadow-x78 <https://github.com/shadow-x78/orbiscreen>
Depends: libgtk-4-1, libadwaita-1-0, libgstreamer1.0-0, libevdev2, libxkbcommon0
Section: utils
Priority: optional
Homepage: https://github.com/shadow-x78/orbiscreen
Description: Real virtual secondary displays for Linux, streamed to Android - over Wi-Fi or USB
 Orbiscreen provides high-performance virtual secondary displays for Linux desktops,
 streaming low-latency video to Android tablets/phones over WebRTC and Wi-Fi/USB.
EOF

dpkg-deb --build "${BUILD_DIR}" "${DEB_NAME}"
echo "==> Debian package built successfully: ${DEB_NAME}"
