#!/usr/bin/env bash
# Orbiscreen - build-appimage (GPL-3.0-or-later)
# https://github.com/shadow-x78/orbiscreen
set -euo pipefail
cd "$(dirname "$0")/../.."
REPO_ROOT="$(pwd)"

if ! command -v cargo >/dev/null 2>&1; then
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
fi

DIST="$REPO_ROOT/dist"
mkdir -p "$DIST"

echo "Building release binary..."
cargo build --release --bin orbiscreen

APP="$DIST/orbiscreen.AppDir"
rm -rf "$APP"
mkdir -p "$APP/usr/bin" "$APP/usr/share/orbiscreen/client/vendor" "$APP/usr/share/icons/hicolor/256x256/apps"

install -m755 target/release/orbiscreen "$APP/usr/bin/orbiscreen"
install -m644 clients/web/index.html "$APP/usr/share/orbiscreen/client/index.html"
install -m644 clients/web/style.css  "$APP/usr/share/orbiscreen/client/style.css"
install -m644 clients/web/app.js     "$APP/usr/share/orbiscreen/client/app.js"
install -m644 clients/web/vendor/mpegts.js "$APP/usr/share/orbiscreen/client/vendor/mpegts.js"

# ---- Bundle GStreamer runtime + plugins -------------------------------
# The daemon links GStreamer and needs at least one H.264 encoder
# (vaapih264enc / nvh264enc / x264enc). We copy the host's GStreamer so the
# AppImage is self-contained; fall back to the host's install if the binary
# can't be located.
GST_PREFIX="$(pkg-config --variable=prefix gstreamer-1.0 2>/dev/null || true)"
if [ -n "${GST_PREFIX:-}" ] && [ -d "$GST_PREFIX/lib" ]; then
    echo "Bundling GStreamer from $GST_PREFIX ..."
    mkdir -p "$APP/usr/lib/gstreamer-1.0"
    GST_LIB_DIR="$(pkg-config --variable=libdir gstreamer-1.0)"
    GST_PLUGIN_DIR="$(pkg-config --variable=pluginsdir gstreamer-1.0)"
    if [ -n "${GST_PLUGIN_DIR}" ] && [ -d "$GST_PLUGIN_DIR/gstreamer-1.0" ]; then
        cp -r "$GST_PLUGIN_DIR/gstreamer-1.0/." "$APP/usr/lib/gstreamer-1.0/"
    fi
    # Copy the GStreamer core/shared libraries the binary links against.
    mkdir -p "$APP/usr/lib"
    for pattern in libgstreamer-1.0 libgstapp-1.0 libgstvideo-1.0 \
                   libgstpbutils-1.0 libgstbase-1.0 libglib-2.0 \
                   libgobject-2.0 libgmodule-2.0 liborc-0.4; do
        find "$GST_LIB_DIR" /usr/lib /usr/lib64 /usr/lib/x86_64-linux-gnu \
             -maxdepth 2 -name "${pattern}.so*" 2>/dev/null -print0 |
        while IFS= read -r -d '' so; do
            cp -f "$so" "$APP/usr/lib/" 2>/dev/null || true
        done
    done
    GST_BUNDLED=1
else
    echo "warning: GStreamer pkg-config prefix not found; AppImage will rely on the host GStreamer install" >&2
    GST_BUNDLED=0
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

if ! command -v magick >/dev/null 2>&1; then
    echo "error: ImageMagick ('magick') is required to rasterize the app icon." >&2
    exit 1
fi
magick -background none "$REPO_ROOT/data/orbiscreen.svg" -flatten -resize 256x256 \
    "$APP/usr/share/icons/hicolor/256x256/apps/orbiscreen.png"
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
    echo "Installing appimagetool into /usr/local/bin..."
    sudo curl -sL \
        https://github.com/AppImageCommunity/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage \
        -o /usr/local/bin/appimagetool
    sudo chmod +x /usr/local/bin/appimagetool
fi

appimagetool "$APP" "$DIST/orbiscreen-x86_64.AppImage"
echo "Built $DIST/orbiscreen-x86_64.AppImage"