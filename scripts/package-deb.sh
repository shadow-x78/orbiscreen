#!/usr/bin/env bash
# Orbiscreen - Debian/Ubuntu (.deb) Package Builder
# https://github.com/shadow-x78/orbiscreen
set -euo pipefail

cd "$(dirname "$0")/.."

VERSION="${1:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')}"
ARCH="amd64"
BUILD_DIR="target/deb-staging"
DEB_NAME="orbiscreen_${VERSION}_${ARCH}.deb"

echo "[Orbiscreen] Building Debian package for Orbiscreen v${VERSION} (${ARCH})..."

if [ ! -f target/release/orbiscreen ]; then
    echo "[Orbiscreen] Building release binaries for deb..."
    cargo build --release --workspace
fi

rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/lib/systemd/user"
mkdir -p "${BUILD_DIR}/usr/share/applications"
mkdir -p "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${BUILD_DIR}/usr/share/orbiscreen/client/vendor"

cp -f target/release/orbiscreen "${BUILD_DIR}/usr/bin/"
cp -f data/orbiscreen.desktop "${BUILD_DIR}/usr/share/applications/"
cp -f data/orbiscreen.svg "${BUILD_DIR}/usr/share/icons/hicolor/scalable/apps/"

cp -f clients/web/index.html "${BUILD_DIR}/usr/share/orbiscreen/client/"
cp -f clients/web/style.css "${BUILD_DIR}/usr/share/orbiscreen/client/"
cp -f clients/web/app.js "${BUILD_DIR}/usr/share/orbiscreen/client/"
cp -rf clients/web/vendor/. "${BUILD_DIR}/usr/share/orbiscreen/client/vendor/"
cp -f scripts/install-evdi-module.sh "${BUILD_DIR}/usr/share/orbiscreen/"

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

cat << EOF > "${BUILD_DIR}/DEBIAN/control"
Package: orbiscreen
Version: ${VERSION}
Architecture: ${ARCH}
Maintainer: shadow-x78 <shadow-x78@users.noreply.github.com>
Depends: libgstreamer1.0-0, libgstreamer-plugins-base1.0-0, gstreamer1.0-plugins-good, gstreamer1.0-plugins-bad, gstreamer1.0-plugins-ugly, gstreamer1.0-libav, libxkbcommon0, libevdev2
Section: utils
Priority: optional
Homepage: https://github.com/shadow-x78/orbiscreen
Description: Turn any Android tablet or phone into a second monitor for Linux
 Orbiscreen turns Android tablets and phones into low-latency (~25ms)
 extended displays for Linux desktops on Wayland and X11. Features native
 graphic tablet digitizer with stylus pressure and tilt for Krita/GIMP,
 auto-orientation, hardware encoding (NVENC/VAAPI), and desktop launcher integration.
EOF

cat <<'EOF' > "${BUILD_DIR}/DEBIAN/prerm"

set -e
if [ "$1" = "remove" ] || [ "$1" = "deconfigure" ]; then

    for u in $(users | tr ' ' '\n' | sort -u); do
        su -s /bin/sh -c "systemctl --user stop orbiscreen || true" "$u" || true
    done
fi
exit 0
EOF
chmod +x "${BUILD_DIR}/DEBIAN/prerm"

cat <<'EOF' > "${BUILD_DIR}/DEBIAN/postrm"

set -e
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    echo "[Orbiscreen] Orbiscreen has been removed."
fi
exit 0
EOF
chmod +x "${BUILD_DIR}/DEBIAN/postrm"

dpkg-deb --build "${BUILD_DIR}" "${DEB_NAME}"
echo "[Orbiscreen] Debian package built successfully: ${DEB_NAME}"
