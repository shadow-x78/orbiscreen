#!/usr/bin/env bash
# Orbiscreen - AppImage Package Builder
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

VERSION="${1:-0.6.0}"
APPDIR="target/AppDir"
APPIMAGE_NAME="orbiscreen-${VERSION}-x86_64.AppImage"

echo "==> Building AppImage for Orbiscreen v${VERSION}..."

mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"

cp -f target/release/orbiscreen "${APPDIR}/usr/bin/"
cp -f target/release/orbiscreen-daemon "${APPDIR}/usr/bin/" || true
cp -f target/release/orbiscreen-gtk "${APPDIR}/usr/bin/" || true

cp -f data/com.orbiscreen.OrbiscreenGtk.desktop "${APPDIR}/com.orbiscreen.OrbiscreenGtk.desktop"
cp -f data/com.orbiscreen.OrbiscreenGtk.desktop "${APPDIR}/usr/share/applications/"
cp -f data/orbiscreen.svg "${APPDIR}/orbiscreen.svg"
cp -f data/orbiscreen.svg "${APPDIR}/.DirIcon"

cat << 'EOF' > "${APPDIR}/AppRun"
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH:-}"
exec "${HERE}/usr/bin/orbiscreen" "$@"
EOF

chmod +x "${APPDIR}/AppRun"
echo "==> AppImage directory staged in ${APPDIR}"
