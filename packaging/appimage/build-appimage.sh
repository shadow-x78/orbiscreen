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
mkdir -p "$APP/usr/bin" "$APP/usr/share/orbiscreen/client" "$APP/usr/share/icons/hicolor/256x256/apps"

install -m755 target/release/orbiscreen "$APP/usr/bin/orbiscreen"
install -m644 clients/web/index.html "$APP/usr/share/orbiscreen/client/index.html"
install -m644 clients/web/style.css  "$APP/usr/share/orbiscreen/client/style.css"
install -m644 clients/web/app.js     "$APP/usr/share/orbiscreen/client/app.js"

cat > "$APP/orbiscreen.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Orbiscreen
GenericName=Virtual Secondary Display
Comment=Stream a virtual display to an Android device
Exec=orbiscreen %u
Icon=orbiscreen
Terminal=true
Categories=Network;System;
StartupNotify=true
EOF

if ! command -v magick >/dev/null 2>&1; then
    echo "error: ImageMagick ('magick') is required to rasterize the app icon." >&2
    exit 1
fi
magick -background white "$REPO_ROOT/data/orbiscreen.svg" -flatten -resize 256x256 \
    "$APP/usr/share/icons/hicolor/256x256/apps/orbiscreen.png"
cp "$APP/usr/share/icons/hicolor/256x256/apps/orbiscreen.png" "$APP/orbiscreen.png"

cat > "$APP/AppRun" <<'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
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