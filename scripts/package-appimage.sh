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
    # shellcheck source=/dev/null
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
GUI_BUILT=0
if command -v cmake >/dev/null 2>&1 && pkg-config --exists Qt6Widgets 2>/dev/null; then
    cmake -S gui -B build-gui -DCMAKE_BUILD_TYPE=Release
    cmake --build build-gui -j"$(nproc)"
    GUI_BUILT=1
else
    echo "warning: Qt 6 Widgets or CMake not found; building without GUI" >&2
fi

# ── Prepare AppDir Staging ──
APP="$DIST/orbiscreen.AppDir"
rm -rf "$APP"
mkdir -p "$APP/usr/bin" "$APP/usr/share/orbiscreen/client" "$APP/usr/share/icons/hicolor"

install -m755 target/release/orbiscreen "$APP/usr/bin/orbiscreen"
if [ "$GUI_BUILT" = 1 ] && [ -f build-gui/orbiscreen-gui ]; then
    install -m755 build-gui/orbiscreen-gui "$APP/usr/bin/orbiscreen-gui"
fi
for f in index.html style.css app.js protocol.js annexb.js stats.js favicon.svg favicon.png apple-touch-icon.png; do
    install -m644 "clients/web/$f" "$APP/usr/share/orbiscreen/client/$f"
done
install -m644 -D clients/web/fonts/noto_kufi_arabic_regular.ttf "$APP/usr/share/orbiscreen/client/fonts/noto_kufi_arabic_regular.ttf"
install -m644 -D clients/web/fonts/noto_kufi_arabic_bold.ttf "$APP/usr/share/orbiscreen/client/fonts/noto_kufi_arabic_bold.ttf"

# ── Runtime dependency helpers ──
# Host libraries that must come from the target system. Everything else that the
# bundled Qt/GStreamer objects link against is copied into the AppDir, so the
# AppImage still works on distroless images (e.g. Bluefin Dakota) that do not
# ship GStreamer plugins or X11 client libraries.
is_host_library() {
    case "$1" in
        ld-linux*|libc.so*|libm.so*|libdl.so*|libpthread.so*|librt.so*|libgcc_s.so*|libstdc++.so*|libresolv.so*|libnss_*|libnss3.so*|libnssutil3.so*|libutil.so*|libcrypt.so*|libanl.so*|libmvec.so*|libgomp.so*)
            return 0 ;;
        libsystemd.so*|libudev.so*|libdbus-1.so*|libpipewire*.so*|libspa*.so*|libselinux.so*|libapparmor.so*|libpam*.so*|libcap.so*|libmount.so*|libblkid.so*|libuuid.so*)
            return 0 ;;
        libglib-2.0.so*|libgobject-2.0.so*|libgmodule-2.0.so*|libz.so*|libzstd.so*|liblzma.so*|liblz4.so*)
            return 0 ;;
        # The host libselinux is loaded against the host pcre2-8; bundling the
        # Ubuntu pcre2-8 produces "no version information available" warnings.
        libpcre2-8.so*)
            return 0 ;;
        libgnutls.so*|libgcrypt.so*|libnettle.so*|libhogweed.so*|libtasn1.so*|libp11-kit.so*|libgssapi*.so*|libkrb5*.so*|libk5crypto*.so*|libkrb5support*.so*|libcom_err*.so*|libkeyutils*.so*)
            return 0 ;;
        libGL*.so*|libEGL*.so*|libOpenGL*.so*|libGLX*.so*|libGLdispatch*.so*|libgbm.so*|libdrm.so*|libwayland*.so*|libxkbcommon.so*)
            return 0 ;;
        libfontconfig.so*|libfreetype.so*|libharfbuzz.so*|libpng*.so*|libjpeg*.so*|libtiff*.so*|libwebp*.so*|libcairo.so*|libpango*.so*|libgdk*.so*|libgtk*.so*)
            return 0 ;;
        libxml2.so*|libxslt*.so*)
            return 0 ;;
    esac
    return 1
}

bundle_library_deps() {
    local target="$1" lib base
    while IFS= read -r lib; do
        [ -n "$lib" ] || continue
        base="$(basename "$lib")"
        [ -e "$APP/usr/lib/$base" ] && continue
        is_host_library "$base" && continue
        cp -Lf "$lib" "$APP/usr/lib/$base" 2>/dev/null || true
    done < <(LD_LIBRARY_PATH="$APP/usr/lib" ldd "$target" 2>/dev/null | awk '/=> \// {print $3}')
}

# ── Bundle GStreamer Runtime ──
GST_PREFIX="$(pkg-config --variable=prefix gstreamer-1.0 2>/dev/null || true)"
if [ -n "${GST_PREFIX:-}" ] && [ -d "$GST_PREFIX/lib" ]; then
    echo "Bundling GStreamer from $GST_PREFIX ..."
    GST_LIB_DIR="$(pkg-config --variable=libdir gstreamer-1.0)"
    GST_PLUGIN_DIR="$(pkg-config --variable=pluginsdir gstreamer-1.0)"
    mkdir -p "$APP/usr/lib" "$APP/usr/lib/gstreamer-1.0" "$APP/usr/libexec/gstreamer-1.0"

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
            cp -Lf "$so" "$APP/usr/lib/" 2>/dev/null || true
        done
    done

    # pkg-config already points at the plugin directory itself on Debian,
    # Ubuntu and Fedora. Appending /gstreamer-1.0 here used to copy zero
    # plugins, leaving the AppImage dependent on host GStreamer.
    if [ -n "${GST_PLUGIN_DIR}" ] && [ -d "$GST_PLUGIN_DIR" ]; then
        echo "Bundling GStreamer plugins from $GST_PLUGIN_DIR ..."
        cp -aL "$GST_PLUGIN_DIR/." "$APP/usr/lib/gstreamer-1.0/"
    else
        echo "error: GStreamer plugin directory not found" >&2
        exit 1
    fi

    plugin_count=0
    while IFS= read -r -d '' plugin; do
        plugin_count=$((plugin_count + 1))
        bundle_library_deps "$plugin"
    done < <(find "$APP/usr/lib/gstreamer-1.0" -maxdepth 1 -type f -name '*.so' -print0)
    if [ "$plugin_count" -eq 0 ]; then
        echo "error: no GStreamer plugins were bundled" >&2
        exit 1
    fi
    echo "Bundled $plugin_count GStreamer plugins"

    GST_SCANNER=""
    for candidate in \
        "$(command -v gst-plugin-scanner 2>/dev/null || true)" \
        "/usr/libexec/gstreamer-1.0/gst-plugin-scanner" \
        "/usr/lib/x86_64-linux-gnu/gstreamer1.0/gst-plugin-scanner" \
        "/usr/lib/gstreamer1.0/gst-plugin-scanner" \
        "$(pkg-config --variable=libexecdir gstreamer-1.0 2>/dev/null)/gstreamer-1.0/gst-plugin-scanner"; do
        if [ -n "$candidate" ] && [ -x "$candidate" ]; then
            GST_SCANNER="$candidate"
            break
        fi
    done
    if [ -n "$GST_SCANNER" ]; then
        install -m755 "$GST_SCANNER" "$APP/usr/libexec/gstreamer-1.0/gst-plugin-scanner"
        bundle_library_deps "$APP/usr/libexec/gstreamer-1.0/gst-plugin-scanner"
    else
        echo "error: gst-plugin-scanner not found; bundled plugins cannot be scanned on a host without GStreamer" >&2
        exit 1
    fi

    missing=0
    while IFS= read -r -d '' plugin; do
        if LD_LIBRARY_PATH="$APP/usr/lib" ldd "$plugin" 2>/dev/null | grep -q "not found"; then
            echo "error: $(basename "$plugin") has unresolved libraries:" >&2
            LD_LIBRARY_PATH="$APP/usr/lib" ldd "$plugin" | grep "not found" >&2
            missing=1
        fi
    done < <(find "$APP/usr/lib/gstreamer-1.0" -maxdepth 1 -type f -name '*.so' -print0)
    [ "$missing" -eq 0 ] || exit 1
else
    echo "warning: GStreamer pkg-config prefix not found; AppImage will rely on the host GStreamer install" >&2
fi

# The daemon links to GStreamer, GLib and xcb directly; pick up anything the
# plugin bundle did not already copy (e.g. libXau/libxcb on a minimal host).
bundle_library_deps "$APP/usr/bin/orbiscreen"

# ── Bundle Qt Runtime ──
# Copy the Qt libraries and every non-host dependency the GUI and its platform
# plugins need (including libxcb/libX11), so the GUI still starts on systems
# that do not install the X11 client stack.
bundle_qt_libraries() {
    local target="$1"
    bundle_library_deps "$target"
}

verify_qt_plugins() {
    local plugin lib missing=0
    while IFS= read -r -d '' plugin; do
        if LD_LIBRARY_PATH="$APP/usr/lib" ldd "$plugin" 2>/dev/null | grep -q "not found"; then
            echo "error: $(basename "$plugin") has unresolved libraries:" >&2
            LD_LIBRARY_PATH="$APP/usr/lib" ldd "$plugin" | grep "not found" >&2
            missing=1
        fi
        while IFS= read -r lib; do
            [ -n "$lib" ] || continue
            local base
            base="$(basename "$lib")"
            if [ ! -e "$APP/usr/lib/$base" ] && ! is_host_library "$base"; then
                echo "error: $base required by $(basename "$plugin") is not bundled" >&2
                missing=1
            fi
        done < <(LD_LIBRARY_PATH="$APP/usr/lib" ldd "$plugin" 2>/dev/null | awk '/=> \// {print $3}')
    done < <(find "$APP/usr/lib/qt6/plugins" -type f -name '*.so' -print0)
    return "$missing"
}

bundle_qt() {
    [ -x "$APP/usr/bin/orbiscreen-gui" ] || return 0
    local qt_plugins
    qt_plugins="$(qtpaths6 --query QT_INSTALL_PLUGINS 2>/dev/null || qmake6 -query QT_INSTALL_PLUGINS 2>/dev/null || true)"
    if [ -n "$qt_plugins" ] && [ -d "$qt_plugins" ]; then
        for dir in platforms styles imageformats \
                   wayland-shell-integration wayland-decoration-client \
                   wayland-graphics-integration-client; do
            if [ "$dir" = "platforms" ]; then
                # The GUI needs exactly two platform plugins: Wayland on a
                # Wayland session and xcb on X11/XWayland. Copying eglfs,
                # linuxfb, vnc, offscreen and vkkhrdisplay pulls in optional
                # host drivers and makes the bundle fragile.
                mkdir -p "$APP/usr/lib/qt6/plugins/platforms"
                for plugin in libqxcb.so libqwayland.so libqwayland-egl.so libqwayland-generic.so; do
                    if [ -f "$qt_plugins/platforms/$plugin" ]; then
                        cp -f "$qt_plugins/platforms/$plugin" "$APP/usr/lib/qt6/plugins/platforms/"
                    fi
                done
            elif [ -d "$qt_plugins/$dir" ]; then
                mkdir -p "$APP/usr/lib/qt6/plugins/$dir"
                cp -r "$qt_plugins/$dir/." "$APP/usr/lib/qt6/plugins/$dir/"
            fi
        done
    else
        echo "warning: Qt plugin directory not found; AppImage will rely on the host Qt 6" >&2
    fi
    mkdir -p "$APP/usr/lib"
    bundle_qt_libraries "$APP/usr/bin/orbiscreen-gui"
    if [ -d "$APP/usr/lib/qt6/plugins" ]; then
        while IFS= read -r -d '' plugin; do
            bundle_qt_libraries "$plugin"
        done < <(find "$APP/usr/lib/qt6/plugins" -type f -name '*.so' -print0)
        verify_qt_plugins
    fi
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
#!/bin/sh
HERE="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
if [ -d "$HERE/usr/lib" ]; then
    export LD_LIBRARY_PATH="$HERE/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi
if [ -d "$HERE/usr/lib/gstreamer-1.0" ]; then
    export GST_PLUGIN_PATH_1_0="$HERE/usr/lib/gstreamer-1.0${GST_PLUGIN_PATH_1_0:+:$GST_PLUGIN_PATH_1_0}"
    # Legacy alias used by some launchers.
    export GST_PLUGIN_PATH="$GST_PLUGIN_PATH_1_0"
    # The bundle is self-contained; do not mix in host plugins built for a
    # different GStreamer version (Bluefin Dakota has no plugins in the base).
    export GST_PLUGIN_SYSTEM_PATH=""
    export GST_PLUGIN_SYSTEM_PATH_1_0=""
    if [ -x "$HERE/usr/libexec/gstreamer-1.0/gst-plugin-scanner" ]; then
        export GST_PLUGIN_SCANNER="$HERE/usr/libexec/gstreamer-1.0/gst-plugin-scanner"
    fi
fi
if [ -d "$HERE/usr/lib/qt6/plugins" ]; then
    export QT_PLUGIN_PATH="$HERE/usr/lib/qt6/plugins${QT_PLUGIN_PATH:+:$QT_PLUGIN_PATH}"
    export QT_QPA_PLATFORM_PLUGIN_PATH="$HERE/usr/lib/qt6/plugins/platforms"
fi
# Let Qt 6 choose the platform plugin (Wayland on a Wayland session, xcb
# otherwise). The bundle now carries the xcb/X11 libraries, so either path works.
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
