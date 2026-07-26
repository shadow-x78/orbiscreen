#!/usr/bin/env bash
# Orbiscreen - Debian/Ubuntu (.deb) Package Builder
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

VERSION="${1:-0.6.0}"
ARCH="amd64"
BUILD_DIR="target/deb-staging"
DEB_NAME="orbiscreen_${VERSION}_${ARCH}.deb"

echo "[Orbiscreen] Building Debian package for Orbiscreen v${VERSION} (${ARCH})..."

mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/share/applications"
mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${BUILD_DIR}/usr/lib/systemd/user"

cp -f target/release/orbiscreen "${BUILD_DIR}/usr/bin/"

cp -f data/com.orbiscreen.OrbiscreenGtk.desktop "${BUILD_DIR}/usr/share/applications/" || true
cp -f data/orbiscreen.svg "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/com.orbiscreen.OrbiscreenGtk.svg" || true

cat << 'EOF' > "${BUILD_DIR}/usr/lib/systemd/user/orbiscreen.service"
[Unit]
Description=Orbiscreen Virtual Secondary Display Service
Documentation=https://github.com/shadow-x78/orbiscreen
After=graphical-session.target

[Service]
Type=exec
ExecStart=/usr/bin/orbiscreen
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

cat <<'EOF' > "${BUILD_DIR}/DEBIAN/prerm"
#!/bin/sh
set -e
if [ "$1" = "remove" ] || [ "$1" = "deconfigure" ]; then
    # Stop the service for all users running it
    for u in $(users); do
        su -s /bin/sh -c "systemctl --user stop orbiscreen || true" "$u"
    done
fi
exit 0
EOF
chmod +x "${BUILD_DIR}/DEBIAN/prerm"

cat <<'EOF' > "${BUILD_DIR}/DEBIAN/postrm"
#!/bin/sh
set -e
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    echo "[Orbiscreen] Orbiscreen has been removed."
fi
exit 0
EOF
chmod +x "${BUILD_DIR}/DEBIAN/postrm"

dpkg-deb --build "${BUILD_DIR}" "${DEB_NAME}"
echo "[Orbiscreen] Debian package built successfully: ${DEB_NAME}"
