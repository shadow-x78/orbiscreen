#!/usr/bin/env bash
# Orbiscreen - AppImage Package Builder
# https://github.com/shadow-x78/orbiscreen

set -euo pipefail

VERSION="${1:-0.6.0}"
APPDIR="target/AppDir"
APPIMAGE_NAME="orbiscreen-${VERSION}-x86_64.AppImage"

echo "[Orbiscreen] Building AppImage for Orbiscreen v${VERSION}..."

mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${APPDIR}/usr/share/orbiscreen/client"

cp -f target/release/orbiscreen "${APPDIR}/usr/bin/"

cp -f data/com.orbiscreen.OrbiscreenGtk.desktop "${APPDIR}/com.orbiscreen.OrbiscreenGtk.desktop"
cp -f data/com.orbiscreen.OrbiscreenGtk.desktop "${APPDIR}/usr/share/applications/"
cp -f data/orbiscreen.svg "${APPDIR}/orbiscreen.svg"
cp -f data/orbiscreen.svg "${APPDIR}/.DirIcon"

cp -f clients/web/index.html "${APPDIR}/usr/share/orbiscreen/client/"
cp -f clients/web/style.css "${APPDIR}/usr/share/orbiscreen/client/"
cp -f clients/web/app.js "${APPDIR}/usr/share/orbiscreen/client/"

cat << 'EOF' > "${APPDIR}/AppRun"
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH:-}"
exec "${HERE}/usr/bin/orbiscreen" "$@"
EOF

chmod +x "${APPDIR}/AppRun"

echo "[Orbiscreen] Downloading appimagetool..."
wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" -O appimagetool
chmod +x appimagetool

echo "[Orbiscreen] Building AppImage file..."
./appimagetool --appimage-extract-and-run "${APPDIR}" "${APPIMAGE_NAME}"

echo "[Orbiscreen] AppImage built successfully: ${APPIMAGE_NAME}"
