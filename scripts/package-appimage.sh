#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Orbiscreen - AppImage Package Builder
# https://github.com/shadow-x78/orbiscreen
# ─────────────────────────────────────────────

# ── Environment & Directory ──
set -euo pipefail

cd "$(dirname "$0")/.."
REPO_ROOT="$(pwd)"

VERSION="${1:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')}"

if ! command -v cargo >/dev/null 2>&1; then
    if [ -f "$HOME/.cargo/env" ]; then
        . "$HOME/.cargo/env"
    else
        echo "error: cargo not found and \$HOME/.cargo/env is missing" >&2
        exit 1
    fi
fi
if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found on PATH" >&2
    exit 1
fi

DIST="$REPO_ROOT/dist"
mkdir -p "$DIST"
ARCH="$(uname -m)"

# ── Build Binaries ──
echo "Building release binaries..."
cargo build --release --workspace
if command -v cmake >/dev/null 2>&1 && pkg-config --exists Qt6Widgets 2>/dev/null; then
    cmake -S gui -B build-gui -DCMAKE_BUILD_TYPE=Release
    cmake --build build-gui -j"$(nproc)"
else
    echo "warning: Qt 6 Widgets or CMake not found; building without GUI" >&2
fi

# ── Prepare AppDir Staging ──
APP="$DIST/orbiscreen.AppDir"
rm -rf "$APP"
mkdir -p "$APP/usr/bin" "$APP/usr/share/orbiscreen/client" "$APP/usr/share/icons/hicolor"

install -m755 target/release/orbiscreen "$APP/usr/bin/orbiscreen"
if [ -f build-gui/orbiscreen-gui ]; then
    install -m755 build-gui/orbiscreen-gui "$APP/usr/bin/orbiscreen-gui"
fi
for f in index.html style.css app.js protocol.js annexb.js stats.js favicon.svg favicon.png apple-touch-icon.png; do
    install -m644 "clients/web/$f" "$APP/usr/share/orbiscreen/client/$f"
done
install -m644 -D clients/web/fonts/noto_kufi_arabic_regular.ttf "$APP/usr/share/orbiscreen/client/fonts/noto_kufi_arabic_regular.ttf"
install -m644 -D clients/web/fonts/noto_kufi_arabic_bold.ttf "$APP/usr/share/orbiscreen/client/fonts/noto_kufi_arabic_bold.ttf"

# ── Bundle GStreamer Runtime ──
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
    LIB_SEARCH_DIRS=()
    for dir in "$GST_LIB_DIR" /usr/lib /usr/lib64 /usr/lib/x86_64-linux-gnu; do
        if [ -d "$dir" ]; then
            LIB_SEARCH_DIRS+=("$dir")
        fi
    done
    for pattern in libgstreamer-1.0 libgstapp-1.0 libgstvideo-1.0 \
                   libgstpbutils-1.0 libgstbase-1.0 libglib-2.0 \
                   libgobject-2.0 libgmodule-2.0 liborc-0.4; do
        find "${LIB_SEARCH_DIRS[@]}" \
             -maxdepth 2 -name "${pattern}.so*" -print0 2>/dev/null |
        while IFS= read -r -d '' so; do
            cp -f "$so" "$APP/usr/lib/" 2>/dev/null || true
        done
    done
else
    echo "warning: GStreamer pkg-config prefix not found; AppImage will rely on the host GStreamer install" >&2
fi

# ── Bundle Qt Runtime ──
bundle_qt() {
    [ -x "$APP/usr/bin/orbiscreen-gui" ] || return 0
    local qt_plugins
    qt_plugins="$(qtpaths6 --query QT_INSTALL_PLUGINS 2>/dev/null || qmake6 -query QT_INSTALL_PLUGINS 2>/dev/null || true)"
    if [ -n "$qt_plugins" ] && [ -d "$qt_plugins" ]; then
        for dir in platforms styles imageformats \
                   wayland-shell-integration wayland-decoration-client \
                   wayland-graphics-integration-client; do
            if [ -d "$qt_plugins/$dir" ]; then
                mkdir -p "$APP/usr/lib/qt6/plugins/$dir"
                cp -r "$qt_plugins/$dir/." "$APP/usr/lib/qt6/plugins/$dir/"
            fi
        done
    else
        echo "warning: Qt plugin directory not found; AppImage will rely on the host Qt 6" >&2
    fi
    mkdir -p "$APP/usr/lib"
    ldd "$APP/usr/bin/orbiscreen-gui" 2>/dev/null |
    awk '/=> \/.*libQt6/ {print $3}' |
    while IFS= read -r lib; do
        [ -n "$lib" ] && cp -f "$lib" "$APP/usr/lib/" 2>/dev/null || true
    done
}
bundle_qt

# ── Desktop & Icon Metadata ──
if [ -f "$APP/usr/bin/orbiscreen-gui" ]; then
    EXEC_LINE="orbiscreen-gui"
    TERMINAL="false"
else
    EXEC_LINE="orbiscreen start"
    TERMINAL="true"
fi
cat > "$APP/orbiscreen.gui.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Orbiscreen
GenericName=Virtual Secondary Display
Comment=Stream a virtual display to an Android device
Exec=${EXEC_LINE}
Icon=orbiscreen
Terminal=${TERMINAL}
Categories=Utility;HardwareSettings;
Keywords=monitor;display;virtual;screen;second;android;
StartupNotify=true
StartupWMClass=orbiscreen.gui
EOF

# Ship the committed hicolor set (rendered at each target size, not downscaled) and fall back to
# rasterizing the 256 entry only if it is missing.
for size in 16 22 24 32 48 64 128 256 512; do
    if [ -f "$REPO_ROOT/gui/icons/${size}x${size}.png" ]; then
        install -Dm644 "$REPO_ROOT/gui/icons/${size}x${size}.png" \
            "$APP/usr/share/icons/hicolor/${size}x${size}/apps/orbiscreen.png"
    fi
done
install -Dm644 "$REPO_ROOT/data/orbiscreen.svg" \
    "$APP/usr/share/icons/hicolor/scalable/apps/orbiscreen.svg"

if [ ! -f "$APP/usr/share/icons/hicolor/256x256/apps/orbiscreen.png" ]; then
    if command -v rsvg-convert >/dev/null 2>&1; then
        rasterize() { rsvg-convert -w 256 -h 256 "$1" -o "$2"; }
    elif command -v magick >/dev/null 2>&1; then
        rasterize() { magick -background none "$1" -resize 256x256 "$2"; }
    elif command -v convert >/dev/null 2>&1; then
        rasterize() { convert -background none "$1" -resize 256x256 "$2"; }
    else
        echo "error: gui/icons/256x256.png is missing and no SVG rasterizer is available" >&2
        exit 1
    fi
    rasterize "$REPO_ROOT/data/orbiscreen.svg" \
        "$APP/usr/share/icons/hicolor/256x256/apps/orbiscreen.png"
fi
cp "$APP/usr/share/icons/hicolor/256x256/apps/orbiscreen.png" "$APP/orbiscreen.png"

# ── AppRun Launcher ──
cat > "$APP/AppRun" <<'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
if [ -d "$HERE/usr/lib/gstreamer-1.0" ]; then
    export GST_PLUGIN_PATH="$HERE/usr/lib/gstreamer-1.0${GST_PLUGIN_PATH:+:$GST_PLUGIN_PATH}"
    export GST_PLUGIN_SCANNER="$HERE/usr/lib/gstreamer-1.0/gst-plugin-scanner"
    export LD_LIBRARY_PATH="$HERE/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi
if [ -d "$HERE/usr/lib/qt6/plugins" ]; then
    export QT_PLUGIN_PATH="$HERE/usr/lib/qt6/plugins${QT_PLUGIN_PATH:+:$QT_PLUGIN_PATH}"
    export QT_QPA_PLATFORM_PLUGIN_PATH="$HERE/usr/lib/qt6/plugins/platforms"
fi
# Launch the desktop GUI when it was bundled (that is what the desktop entry and the icon set are
# for); fall back to the CLI only in GUI-less builds.
if [ -x "$HERE/usr/bin/orbiscreen-gui" ] && [ -z "${ORBISCREEN_CLI:-}" ]; then
    exec "$HERE/usr/bin/orbiscreen-gui" "$@"
fi
exec "$HERE/usr/bin/orbiscreen" "$@"
EOF
chmod +x "$APP/AppRun"

# ── Build AppImage ──
if ! command -v appimagetool >/dev/null 2>&1; then
    APPIMAGETOOL_VERSION="1.9.1"
    APPIMAGETOOL_URL="https://github.com/AppImage/appimagetool/releases/download/${APPIMAGETOOL_VERSION}/appimagetool-${ARCH}.AppImage"
    mkdir -p "$DIST/tools"
    echo "Downloading appimagetool ${APPIMAGETOOL_VERSION}..."
    curl -fsSL "$APPIMAGETOOL_URL" -o "$DIST/tools/appimagetool.AppImage"
    case "$ARCH" in
        x86_64)
            APPIMAGETOOL_SHA256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0"
            ;;
        *)
            echo "error: no pinned SHA256 for appimagetool on ${ARCH}; refusing to execute an" >&2
            echo "       unverified download. Install appimagetool manually and re-run." >&2
            exit 1
            ;;
    esac
    echo "${APPIMAGETOOL_SHA256}  $DIST/tools/appimagetool.AppImage" | sha256sum -c -
    chmod +x "$DIST/tools/appimagetool.AppImage"
    PATH="$DIST/tools:$PATH"
    ln -sf "$DIST/tools/appimagetool.AppImage" "$DIST/tools/appimagetool"
    export APPIMAGE_EXTRACT_AND_RUN=1
fi

APPIMAGE_PATH="$DIST/orbiscreen-${VERSION}-${ARCH}.AppImage"
if ! appimagetool -u "gh-releases-zsync|shadow-x78|orbiscreen|latest|orbiscreen-*${ARCH}.AppImage.zsync" "$APP" "$APPIMAGE_PATH"; then
    echo "error: appimagetool failed" >&2
    exit 1
fi

echo "Built $APPIMAGE_PATH"
