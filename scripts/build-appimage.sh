#!/usr/bin/env bash
# Orbiscreen - AppImage Package Builder (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$(pwd)"

if ! command -v cargo >/dev/null 2>&1; then
    . "$HOME/.cargo/env"
fi

DIST="$REPO_ROOT/dist"
mkdir -p "$DIST"

echo "Building release binaries..."
cargo build --release --workspace

APP="$DIST/orbiscreen.AppDir"
rm -rf "$APP"
mkdir -p "$APP/usr/bin" "$APP/usr/share/orbiscreen/client/vendor" "$APP/usr/share/icons/hicolor/256x256/apps"

install -m755 target/release/orbiscreen "$APP/usr/bin/orbiscreen"
install -m755 target/release/orbiscreen-gtk "$APP/usr/bin/orbiscreen-gtk"
install -m644 clients/web/index.html "$APP/usr/share/orbiscreen/client/index.html"
install -m644 clients/web/style.css  "$APP/usr/share/orbiscreen/client/style.css"
install -m644 clients/web/app.js     "$APP/usr/share/orbiscreen/client/app.js"
install -m644 clients/web/vendor/mpegts.js "$APP/usr/share/orbiscreen/client/vendor/mpegts.js"

GST_PREFIX="$(pkg-config --variable=prefix gstreamer-1.0 2>/dev/null || true)"
if [ -n "${GST_PREFIX:-}" ] && [ -d "$GST_PREFIX/lib" ]; then
    echo "Bundling GStreamer from $GST_PREFIX ..."
    mkdir -p "$APP/usr/lib/gstreamer-1.0"
    GST_LIB_DIR="$(pkg-config --variable=libdir gstreamer-1.0)"
    GST_PLUGIN_DIR="$(pkg-config --variable=pluginsdir gstreamer-1.0)"
    if [ -n "${GST_PLUGIN_DIR}" ] && [ -d "$GST_PLUGIN_DIR/gstreamer-1.0" ]; then
        cp -r "$GST_PLUGIN_DIR/gstreamer-1.0/." "$APP/usr/lib/gstreamer-1.0/"
    fi
    mkdir -p "$APP/usr/lib"
    for pattern in libgstreamer-1.0 libgstapp-1.0 libgstvideo-1.0 \
                   libgstpbutils-1.0 libgstbase-1.0 libglib-2.0 \
                   libgobject-2.0 libgmodule-2.0 liborc-0.4; do
        find "$GST_LIB_DIR" /usr/lib /usr/lib64 /usr/lib/x86_64-linux-gnu \
             -maxdepth 2 -name "${pattern}.so*" -print0 2>/dev/null |
        while IFS= read -r -d '' so; do
            cp -f "$so" "$APP/usr/lib/" 2>/dev/null || true
        done
    done
else
    echo "warning: GStreamer pkg-config prefix not found; AppImage will rely on the host GStreamer install" >&2
fi

cat > "$APP/orbiscreen.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Orbiscreen
GenericName=Virtual Secondary Display
Comment=Stream a virtual display to an Android device
Exec=orbiscreen start
Icon=orbiscreen
Terminal=true
Categories=Network;System;
StartupNotify=true
EOF

rasterize_256() {
    local svg="$1" out="$2"
    if command -v rsvg-convert >/dev/null 2>&1; then
        rsvg-convert -w 256 -h 256 "$svg" -o "$out"
    elif command -v magick >/dev/null 2>&1; then
        magick -background none "$svg" -flatten -resize 256x256 "$out"
    elif command -v convert >/dev/null 2>&1; then
        convert -background none "$svg" -flatten -resize 256x256 "$out"
    else
        echo "error: no SVG rasterizer found (need 'rsvg-convert', 'magick' or 'convert')" >&2
        exit 1
    fi
}
rasterize_256 "$REPO_ROOT/data/orbiscreen.svg" "$APP/usr/share/icons/hicolor/256x256/apps/orbiscreen.png"
cp "$APP/usr/share/icons/hicolor/256x256/apps/orbiscreen.png" "$APP/orbiscreen.png"

cat > "$APP/AppRun" <<'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
if [ -d "$HERE/usr/lib/gstreamer-1.0" ]; then
    export GST_PLUGIN_PATH="$HERE/usr/lib/gstreamer-1.0${GST_PLUGIN_PATH:+:$GST_PLUGIN_PATH}"
    export GST_PLUGIN_SCANNER="$HERE/usr/lib/gstreamer-1.0/gst-plugin-scanner"
    export LD_LIBRARY_PATH="$HERE/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi
exec "$HERE/usr/bin/orbiscreen" "$@"
EOF
chmod +x "$APP/AppRun"

if ! command -v appimagetool >/dev/null 2>&1; then
    APPIMAGETOOL_VERSION="1.9.1"
    APPIMAGETOOL_SHA256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0"
    APPIMAGETOOL_URL="https://github.com/AppImage/appimagetool/releases/download/${APPIMAGETOOL_VERSION}/appimagetool-x86_64.AppImage"
    mkdir -p "$DIST/tools"
    echo "Downloading appimagetool ${APPIMAGETOOL_VERSION}..."
    curl -fsSL "$APPIMAGETOOL_URL" -o "$DIST/tools/appimagetool.AppImage"
    echo "${APPIMAGETOOL_SHA256}  $DIST/tools/appimagetool.AppImage" | sha256sum -c -
    chmod +x "$DIST/tools/appimagetool.AppImage"
    PATH="$DIST/tools:$PATH"
    ln -sf "$DIST/tools/appimagetool.AppImage" "$DIST/tools/appimagetool"
    export APPIMAGE_EXTRACT_AND_RUN=1
fi

appimagetool "$APP" "$DIST/orbiscreen-x86_64.AppImage"
echo "Built $DIST/orbiscreen-x86_64.AppImage"
